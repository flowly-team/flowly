use std::pin::{Pin, pin};

use futures::{Stream, StreamExt, stream::poll_fn};
use tokio::sync::mpsc;

use crate::Service;

pub struct Spawn<S> {
    pub(crate) buffer: usize,
    pub(crate) service: S,
}

impl<I: Send, S: Service<I> + Send + Sync + 'static> Service<I> for Spawn<S>
where
    S::Out: Send,
{
    type Out = S::Out;

    #[inline]
    fn handle(self, input: impl Stream<Item = I> + Send) -> impl Stream<Item = S::Out> + Send {
        let (tx, mut rx) = mpsc::channel(self.buffer);

        let fut = Box::pin(async move {
            let mut stream = pin!(self.service.handle(input));

            while let Some(item) = stream.next().await {
                if (tx.send(item).await).is_err() {
                    break;
                }
            }
        }) as Pin<Box<dyn Future<Output = ()> + Send>>;

        // SAFTY:
        // This is safe because:
        //  - input stream will be dropped as soon as it drained.
        //  - it is garateed that input lives as long as it contains items
        tokio::spawn(unsafe {
            std::mem::transmute::<
                Pin<Box<dyn Future<Output = ()> + Send>>,
                Pin<Box<dyn Future<Output = ()> + Send + 'static>>,
            >(fut)
        });

        poll_fn(move |cx| rx.poll_recv(cx))
    }
}
