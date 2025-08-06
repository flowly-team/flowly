use futures::Stream;

use crate::{Context, Service};

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
