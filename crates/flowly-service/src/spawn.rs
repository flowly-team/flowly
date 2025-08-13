use std::{marker::PhantomData, pin::pin};

use futures::{Stream, StreamExt};
use tokio::sync::mpsc;

use crate::{Context, Service};

struct SpawnEachTask<I: Send, S: Service<I>> {
    tx: flowly_spsc::Sender<I>,
    handle: tokio::task::JoinHandle<()>,
    m: PhantomData<S>,
}

impl<I, S> SpawnEachTask<I, S>
where
    S::Out: Send + 'static,
    I: Send + 'static,
    S: Service<I> + Send + 'static,
{
    fn new(mut s: S, out_tx: mpsc::Sender<S::Out>, cx: Context) -> Self {
        let (tx, mut rx) = flowly_spsc::channel(1);

        Self {
            tx,
            handle: tokio::spawn(async move {
                while let Some(item) = rx.recv().await {
                    let mut s = pin!(s.handle(item, &cx));
                    while let Some(x) = s.next().await {
                        out_tx.send(x).await;
                    }
                }
            }),
            m: PhantomData,
        }
    }
}

pub struct SpawnEach<I: Send + 'static, S: Service<I>> {
    service: S,
    tasks: Vec<SpawnEachTask<I, S>>,
}

impl<I, S> SpawnEach<I, S>
where
    I: Send,
    S: Service<I>,
{
    pub(crate) fn new(service: S) -> Self {
        Self {
            service,
            tasks: Vec::new(),
        }
    }
}

impl<I, S> Service<I> for SpawnEach<I, S>
where
    I: Send,
    S: Service<I> + Send,
    S::Out: Send,
{
    type Out = S::Out;

    fn handle(&mut self, input: I, cx: &Context) -> impl Stream<Item = Self::Out> + Send {
        futures::stream::empty()
    }
}

pub struct Spawn<S> {
    pub(crate) service: S,
}

pub struct SpawnLocal<S> {
    pub(crate) service: S,
}

impl<I: Send, S: Service<I> + Send + 'static> Service<I> for Spawn<S>
where
    S::Out: Send,
{
    type Out = S::Out;

    #[inline]
    fn handle(&mut self, input: I, cx: &Context) -> impl Stream<Item = S::Out> {
        let _ = cx;
        let _ = input;

        unimplemented!();

        futures::stream::empty()
        // let (tx, mut rx) = mpsc::channel(self.buffer);

        // let fut = Box::pin(async move {
        //     let mut stream = pin!(self.service.handle(input));

        //     while let Some(item) = stream.next().await {
        //         if (tx.send(item).await).is_err() {
        //             break;
        //         }
        //     }
        // }) as Pin<Box<dyn Future<Output = ()> + Send>>;

        // // SAFTY:
        // // This is safe because:
        // //  - input stream will be dropped as soon as it drained.
        // //  - it is garateed that input lives as long as it contains items
        // tokio::spawn(unsafe {
        //     std::mem::transmute::<
        //         Pin<Box<dyn Future<Output = ()> + Send>>,
        //         Pin<Box<dyn Future<Output = ()> + Send + 'static>>,
        //     >(fut)
        // });

        // poll_fn(move |cx| rx.poll_recv(cx))
    }
}
