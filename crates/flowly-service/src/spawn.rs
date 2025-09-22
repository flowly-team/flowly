use std::{marker::PhantomData, pin::pin};

use futures::{FutureExt, Stream, StreamExt};
use tokio::sync::{Mutex, mpsc};

use crate::{Context, Service};

const MAX_SPAWN_TASKS: usize = 256;

struct SpawnEachTask<I: Send, S: Service<I>> {
    id: u32,
    tx: flowly_spsc::Sender<I>,
    m: PhantomData<S>,
    _handle: tokio::task::JoinHandle<()>,
}

impl<I, S> SpawnEachTask<I, S>
where
    S::Out: Send + 'static,
    I: Send + 'static,
    S: Service<I> + Send + 'static,
{
    fn new(
        id: u32,
        buffer: usize,
        mut s: S,
        out_tx: mpsc::Sender<Option<S::Out>>,
        cx: Context,
        input: I,
    ) -> Self {
        let (mut tx, mut rx) = flowly_spsc::channel(buffer);

        let _handle = tokio::spawn(async move {
            'recv: while let Some(item) = rx.recv().await {
                println!("[{id}] got item");

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

        tx.try_send(input).unwrap();

        Self {
            id,
            tx,
            _handle,
            m: PhantomData,
        }
    }

    #[inline]
    async fn send(&mut self, input: I) -> Result<(), flowly_spsc::TrySendError<I>> {
        self.tx.send(input).await
    }
}

pub struct SpawnEach<I: Send + 'static, S: Service<I>> {
    service: S,
    sender: mpsc::Sender<Option<S::Out>>,
    receiver: Mutex<mpsc::Receiver<Option<S::Out>>>,
    tasks: Vec<SpawnEachTask<I, S>>,
    _m: PhantomData<I>,
    counter: u32,
}

impl<I, S> SpawnEach<I, S>
where
    I: Send,
    S: Service<I> + Send,
    S::Out: Send,
{
    pub(crate) fn new(service: S) -> Self {
        let (sender, rx) = mpsc::channel(1);

        Self {
            service,
            sender,
            receiver: Mutex::new(rx),
            tasks: Vec::with_capacity(MAX_SPAWN_TASKS),
            _m: PhantomData,
            counter: 0,
        }
    }

    #[inline]
    fn drain_rx(&mut self) -> impl Stream<Item = S::Out> + Send {
        async_stream::stream! {
            let mut guard = self.receiver.lock().await;
            while let Some(res) = guard.recv().await {
                if let Some(item) = res {
                    yield item;
                } else {
                    break;
                }
            }
        }
    }
}

impl<I, S> Service<I> for SpawnEach<I, S>
where
    I: Send,
    S: Service<I> + Clone + Send + 'static,
    S::Out: Send,
{
    type Out = S::Out;

    fn handle(&mut self, mut input: I, cx: &Context) -> impl Stream<Item = Self::Out> + Send {
        let index = if self.tasks.is_empty() {
            0
        } else {
            fastrand::usize(0..self.tasks.len())
        };

        let (left, right) = self.tasks.split_at_mut(index);

        for task in right.iter_mut().chain(left.iter_mut()) {
            if let Err(err) = task.tx.try_send(input) {
                input = err.val;
            } else {
                return self.drain_rx().right_stream();
            }
        }

        if self.tasks.len() < MAX_SPAWN_TASKS {
            self.tasks.push(SpawnEachTask::new(
                self.counter,
                2,
                self.service.clone(),
                self.sender.clone(),
                cx.clone(),
                input,
            ));

            self.counter += 1;
            self.drain_rx().right_stream()
        } else {
            async move {
                if self.tasks[index].send(input).await.is_err() {
                    log::error!("cannot send the message. channel closed!");
                }

                self.drain_rx()
            }
            .into_stream()
            .flatten()
            .left_stream()
        }
    }
}

pub fn spawn_each<I, S>(service: S) -> SpawnEach<I, S>
where
    I: Send,
    S: Send + Service<I> + Clone + 'static,
    S::Out: Send,
{
    SpawnEach::new(service)
}

// pub struct Spawn<S> {
//     pub(crate) service: S,
// }

// pub struct SpawnLocal<S> {
//     pub(crate) service: S,
// }

// impl<I: Send, S: Service<I> + Send + 'static> Service<I> for Spawn<S>
// where
//     S::Out: Send,
// {
//     type Out = S::Out;

//     #[inline]
//     fn handle(&mut self, input: I, cx: &Context) -> impl Stream<Item = S::Out> {
//         let _ = cx;
//         let _ = input;

//         unimplemented!();

//         futures::stream::empty()
//         // let (tx, mut rx) = mpsc::channel(self.buffer);

//         // let fut = Box::pin(async move {
//         //     let mut stream = pin!(self.service.handle(input));

//         //     while let Some(item) = stream.next().await {
//         //         if (tx.send(item).await).is_err() {
//         //             break;
//         //         }
//         //     }
//         // }) as Pin<Box<dyn Future<Output = ()> + Send>>;

//         // // SAFTY:
//         // // This is safe because:
//         // //  - input stream will be dropped as soon as it drained.
//         // //  - it is garateed that input lives as long as it contains items
//         // tokio::spawn(unsafe {
//         //     std::mem::transmute::<
//         //         Pin<Box<dyn Future<Output = ()> + Send>>,
//         //         Pin<Box<dyn Future<Output = ()> + Send + 'static>>,
//         //     >(fut)
//         // });

//         // poll_fn(move |cx| rx.poll_recv(cx))
//     }
// }
