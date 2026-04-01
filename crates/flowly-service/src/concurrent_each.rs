use std::{
    marker::PhantomData,
    pin::{Pin, pin},
    sync::Arc,
    task::{Poll, ready},
};

use futures::{FutureExt, Stream, StreamExt};
use tokio::sync::{Mutex, OwnedMutexGuard};

use crate::{Context, Service};

pub struct ConcurrentRx<T: Send> {
    guard: OwnedMutexGuard<flowly_spsc::Receiver<Option<T>>>,
}

impl<T: Send> Stream for ConcurrentRx<T> {
    type Item = T;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        match ready!(self.guard.poll_recv(cx)) {
            Some(Some(val)) => Poll::Ready(Some(val)),
            Some(None) => Poll::Ready(None),
            None => Poll::Ready(None),
        }
    }
}

struct ConcurrentTask<I: Send, S: Service<I>> {
    #[allow(dead_code)]
    id: u32,
    tx: tokio::sync::mpsc::Sender<I>,
    rx: Arc<Mutex<flowly_spsc::Receiver<Option<S::Out>>>>,
    _handle: tokio::task::JoinHandle<()>,
    m: PhantomData<S>,
    ctx_tx: std::sync::Mutex<Option<tokio::sync::oneshot::Sender<Context>>>,
}

impl<I, S> ConcurrentTask<I, S>
where
    S::Out: Send + 'static,
    I: Send + 'static,
    S: Service<I> + Send + 'static,
{
    fn new(id: u32, s: Arc<S>) -> Self {
        let (tx, mut in_rx) = tokio::sync::mpsc::channel(1);
        let (mut out_tx, out_rx) = flowly_spsc::channel(1);
        let (ctx_tx, ctx_rx) = tokio::sync::oneshot::channel();

        let _handle = tokio::spawn(async move {
            let Ok(cx) = ctx_rx.await else {
                log::error!("no context got");
                return;
            };

            'recv: while let Some(item) = in_rx.recv().await {
                let mut s = pin!(s.handle(item, &cx));

                while let Some(x) = s.next().await {
                    if out_tx.send(Some(x)).await.is_err() {
                        log::error!("cannot send the message. channel closed!");
                        break 'recv;
                    }
                }

                if out_tx.send(None).await.is_err() {
                    log::error!("cannot send the message. channel closed!");
                    break 'recv;
                }
            }
        });

        Self {
            id,
            tx,
            ctx_tx: std::sync::Mutex::new(Some(ctx_tx)),
            rx: Arc::new(tokio::sync::Mutex::new(out_rx)),
            _handle,
            m: PhantomData,
        }
    }

    #[inline]
    fn is_available(&self) -> bool {
        self.rx.try_lock().is_ok()
    }

    #[inline]
    async fn send(
        &self,
        input: I,
    ) -> Result<ConcurrentRx<S::Out>, tokio::sync::mpsc::error::SendError<I>> {
        self.tx.send(input).await?;

        Ok(ConcurrentRx {
            guard: self.rx.clone().lock_owned().await,
        })
    }

    fn is_ready(&self) -> bool {
        if let Ok(lock) = self.ctx_tx.try_lock() {
            lock.is_none()
        } else {
            false
        }
    }

    fn init(&self, ctx: Context) {
        if let Ok(Some(sender)) = self.ctx_tx.try_lock().map(|mut x| x.take()) {
            if sender.send(ctx).is_err() {
                log::warn!("cannot send context: receiver closed");
            }
        } else {
            log::warn!("cannot init ConcurrentTask twice");
        }
    }
}

pub struct ConcurrentEach<I: Send + 'static, S: Service<I>> {
    service: Arc<S>,
    tasks: Vec<ConcurrentTask<I, S>>,
    _m: PhantomData<I>,
    limit: usize,
}

impl<I: Send + 'static + Clone, S: Service<I> + Clone> Clone for ConcurrentEach<I, S> {
    fn clone(&self) -> Self {
        Self {
            service: self.service.clone(),
            tasks: Vec::new(),
            _m: self._m,
            limit: self.limit,
        }
    }
}

impl<I, S> ConcurrentEach<I, S>
where
    I: Send,
    S: Service<I> + Send + 'static,
    S::Out: Send,
{
    pub fn new(service: S, limit: usize) -> Self {
        let service = Arc::new(service);
        Self {
            tasks: (0..limit as u32)
                .map(|id| ConcurrentTask::new(id, service.clone()))
                .collect(),
            service,
            _m: PhantomData,
            limit,
        }
    }
}

impl<I, R, E, S> Service<I> for ConcurrentEach<I, S>
where
    I: Send + Sync,
    R: Send + 'static,
    E: Send + 'static,
    S: Service<I, Out = Result<R, E>> + Clone + Send + 'static,
{
    type Out = Result<ConcurrentRx<S::Out>, E>;

    fn handle(&self, input: I, cx: &Context) -> impl Stream<Item = Self::Out> + Send {
        async move {
            let mut index = fastrand::usize(0..self.tasks.len());

            for idx in 0..self.tasks.len() {
                let idx = (idx + self.tasks.len()) % self.tasks.len();
                if self.tasks[idx].is_available() {
                    index = idx;
                    break;
                }
            }

            if !self.tasks[index].is_ready() {
                self.tasks[index].init(cx.clone());
            }

            Ok(self.tasks[index].send(input).await.unwrap())
        }
        .into_stream()
    }
}

pub fn concurrent_each<I, S>(service: S, limit: usize) -> ConcurrentEach<I, S>
where
    I: Send,
    S: Send + Service<I> + Clone + 'static,
    S::Out: Send,
{
    ConcurrentEach::new(service, limit)
}
