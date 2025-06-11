mod atomic_waker;
mod error;

use std::{
    cell::UnsafeCell,
    future::poll_fn,
    mem::MaybeUninit,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    task::{Context, Poll},
};

use atomic_waker::AtomicWaker;
pub use error::{SendError, TryRecvError, TrySendError};
use futures::Stream;

pub fn channel<T>(size: usize) -> (Sender<T>, Receiver<T>) {
    let shared = Arc::new(Shared::new(size));

    (
        Sender {
            shared: shared.clone(),
            pos: 0,
        },
        Receiver { shared, pos: 0 },
    )
}

struct Shared<T> {
    buf: Box<[UnsafeCell<MaybeUninit<T>>]>,
    consumer: AtomicWaker,
    producer: AtomicWaker,
    count: AtomicUsize,
    closed: AtomicBool,
    capacity: usize,
}

unsafe impl<T: Send> Send for Shared<T> {}
unsafe impl<T: Sync> Sync for Shared<T> {}

impl<T> Shared<T> {
    pub(crate) fn new(capacity: usize) -> Self {
        let capacity = std::cmp::max(capacity + 1, 2);
        let buf = (0..capacity)
            .map(|_| UnsafeCell::new(MaybeUninit::uninit()))
            .collect();

        Self {
            buf,
            consumer: Default::default(),
            producer: Default::default(),
            closed: AtomicBool::new(false),
            count: AtomicUsize::new(0),
            capacity,
        }
    }

    #[inline]
    pub(crate) fn index(&self, index: usize) -> usize {
        index % self.capacity
    }

    #[inline]
    pub(crate) fn len(&self) -> usize {
        self.count.load(Ordering::Relaxed)
    }

    #[inline]
    pub(crate) fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub(crate) fn is_full(&self) -> bool {
        self.len() == self.capacity
    }

    #[inline]
    pub(crate) unsafe fn get_unchecked(&self, idx: usize) -> T {
        let ptr = self.buf.as_ptr();

        unsafe { (&*ptr.add(idx)).get().read().assume_init() }
    }

    #[inline]
    pub(crate) unsafe fn set_unchecked(&self, idx: usize, value: T) {
        unsafe {
            self.buf
                .get_unchecked(idx)
                .get()
                .write(MaybeUninit::new(value))
        };
    }
}

pub struct Sender<T> {
    shared: Arc<Shared<T>>,
    pos: usize,
}

impl<T> Sender<T> {
    /// Returns whether this channel is closed.
    #[inline]
    pub fn is_closed(&self) -> bool {
        self.shared.closed.load(Ordering::Relaxed)
    }

    pub fn start_send(&mut self, item: T) -> Result<(), TrySendError<T>> {
        if self.is_closed() {
            return Err(TrySendError {
                err: SendError::Disconnected,
                val: item,
            });
        }

        if let Some(idx) = self.next_idx() {
            unsafe {
                self.shared.set_unchecked(idx, item);
            }

            Ok(())
        } else {
            Err(TrySendError {
                err: SendError::Full,
                val: item,
            })
        }
    }

    #[inline]
    pub fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), SendError>> {
        if self.shared.is_full() {
            self.poll_flush(cx)
        } else {
            Poll::Ready(Ok(()))
        }
    }

    pub fn poll_flush(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), SendError>> {
        if self.is_closed() {
            Poll::Ready(Err(SendError::Disconnected))
        } else if self.shared.is_empty() {
            // if the inner bounded is already empty,
            // we just return ok to avoid some atomic operation.
            Poll::Ready(Ok(()))
        } else {
            self.shared.producer.register(cx.waker());
            self.shared.consumer.wake();
            Poll::Pending
        }
    }

    #[inline]
    pub async fn flush(&mut self) -> Result<(), SendError> {
        poll_fn(|cx| self.poll_flush(cx)).await
    }

    pub async fn send(&mut self, item: T) -> Result<(), TrySendError<T>> {
        let idx = match poll_fn(|cx| self.poll_next_pos(cx)).await {
            Ok(idx) => idx,
            Err(err) => return Err(TrySendError { err, val: item }),
        };

        unsafe {
            self.shared.set_unchecked(idx, item);
        }

        self.shared.consumer.wake();

        Ok(())
    }

    fn poll_next_pos(&mut self, cx: &mut Context<'_>) -> Poll<Result<usize, SendError>> {
        if self.is_closed() {
            return Poll::Ready(Err(SendError::Disconnected));
        }

        if let Some(idx) = self.next_idx() {
            Poll::Ready(Ok(idx))
        } else {
            self.shared.producer.register(cx.waker());

            // We need to poll again, in case of the receiver take some items during
            // the register and the previous poll
            if let Some(idx) = self.next_idx() {
                Poll::Ready(Ok(idx))
            } else {
                Poll::Pending
            }
        }
    }

    #[inline]
    fn next_idx(&mut self) -> Option<usize> {
        if self.shared.is_full() {
            None
        } else {
            let idx = self.pos;
            self.pos += 1;
            self.shared.count.fetch_add(1, Ordering::Relaxed);
            Some(self.shared.index(idx))
        }
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        // we need to wake up the receiver before
        // the sender was totally dropped, otherwise the receiver may hang up.
        self.shared.closed.store(true, Ordering::Relaxed);
        self.shared.consumer.wake();
    }
}

pub struct Receiver<T> {
    shared: Arc<Shared<T>>,
    pos: usize,
}

impl<T> Receiver<T> {
    pub fn try_recv(&mut self) -> Result<Option<T>, TryRecvError> {
        match self.try_pop() {
            None => {
                // If there is no item in this bounded, we need to
                // check closed and try pop again.
                //
                // Consider this situation:
                // receiver try pop first, and sender send an item then close.
                // If we just check closed without pop again, the remaining item will be lost.
                if self.is_closed() {
                    match self.try_pop() {
                        None => Err(TryRecvError::Disconnected),
                        Some(item) => Ok(Some(item)),
                    }
                } else {
                    Ok(None)
                }
            }
            Some(item) => Ok(Some(item)),
        }
    }

    pub fn poll_want_recv(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        if self.is_closed() {
            return Poll::Ready(());
        }

        self.shared.consumer.register(cx.waker());
        self.shared.producer.wake();

        if self.shared.is_empty() {
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }

    #[inline]
    pub async fn want_recv(&mut self) {
        poll_fn(|cx| self.poll_want_recv(cx)).await
    }

    pub fn poll_recv(&mut self, cx: &mut Context<'_>) -> Poll<Option<T>> {
        if let Poll::Ready(op) = self.poll_next_msg() {
            return Poll::Ready(Some(op));
        }

        self.shared.consumer.register(cx.waker());

        // 1. We need to poll again,
        //    in case of some item was sent between the registering and the previous poll.
        //
        // 2. We need to see whether this channel is closed. Because the sender could
        //    be closed and wake receiver before the register operation, so if we don't check close,
        //    this method may return Pending and will never be wakeup.
        if self.is_closed() {
            match self.poll_next_msg() {
                Poll::Ready(op) => Poll::Ready(Some(op)),
                Poll::Pending => Poll::Ready(None),
            }
        } else {
            self.poll_next_msg().map(Some)
        }
    }

    #[inline]
    pub async fn recv(&mut self) -> Option<T> {
        poll_fn(|cx| self.poll_recv(cx)).await
    }

    #[inline]
    pub fn is_closed(&self) -> bool {
        self.shared.closed.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn close(&mut self) {
        self.shared.closed.store(true, Ordering::Relaxed)
    }

    fn poll_next_msg(&mut self) -> Poll<T> {
        match self.try_pop() {
            None => Poll::Pending,
            Some(item) => {
                self.shared.producer.wake();
                Poll::Ready(item)
            }
        }
    }

    pub(crate) fn try_pop(&mut self) -> Option<T> {
        if self.shared.is_empty() {
            None
        } else {
            unsafe {
                let now = self.pos;
                let idx = self.shared.index(now);
                self.pos = now + 1;
                self.shared.count.fetch_sub(1, Ordering::Relaxed);
                Some(self.shared.get_unchecked(idx))
            }
        }
    }
}

impl<T> Stream for Receiver<T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.poll_recv(cx)
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        self.close();
    }
}
