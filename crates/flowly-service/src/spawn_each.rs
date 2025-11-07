use std::{marker::PhantomData, pin::pin};

use futures::StreamExt;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use crate::Service;

pub struct SpawnEach<M, S> {
    service: S,
    buffer: usize,
    _m: PhantomData<M>,
}

impl<M, S> SpawnEach<M, S>
where
    M: Send,
    S: Service<M> + Send + Clone + 'static,
{
    pub fn new(service: S, buffer: usize) -> Self
    where
        S::Out: Send,
    {
        Self {
            service,
            buffer,
            _m: PhantomData,
        }
    }
}

impl<M, R, E, S> Service<M> for SpawnEach<M, S>
where
    M: Send + 'static,
    R: Send + 'static,
    E: Send + 'static,
    S: Clone + Service<M, Out = Result<R, E>> + Send + 'static,
{
    type Out = Result<ReceiverStream<Result<R, E>>, E>;

    fn handle(
        &mut self,
        input: M,
        cx: &crate::Context,
    ) -> impl futures::Stream<Item = Self::Out> + Send {
        let mut service = self.service.clone();
        let (tx, rx) = mpsc::channel(self.buffer);
        let cx = cx.clone();
        tokio::spawn(async move {
            let mut stream = pin!(service.handle(input, &cx));

            while let Some(Some(res)) = cx.fuse_abort(stream.next()).await {
                if tx.send(res).await.is_err() {
                    log::warn!("mpsc::send failed");
                    break;
                }
            }
        });

        futures::stream::iter([Ok(ReceiverStream::new(rx))])
    }
}
pub fn spawn_each<M, S>(service: S) -> SpawnEach<M, S>
where
    M: Send,
    S: Send + Service<M> + Clone + 'static,
    S::Out: Send,
{
    SpawnEach::new(service, 2)
}
