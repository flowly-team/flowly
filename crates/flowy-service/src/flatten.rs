use std::pin::pin;

use futures::{Stream, StreamExt};

use crate::Service;

pub struct TryFlattenMap<F> {
    pub(crate) f: F,
}

impl<I, G, Ft, U, E, F> Service<Result<I, E>> for TryFlattenMap<F>
where
    G: Send,
    E: Send,
    I: Send,
    Ft: Future<Output = Result<U, E>> + Send,
    F: FnMut(I) -> Ft + Send,
    U: Stream<Item = Result<G, E>> + Send,
{
    type Out = Result<G, E>;

    fn handle(
        mut self,
        input: impl Stream<Item = Result<I, E>> + Send,
    ) -> impl Stream<Item = Result<G, E>> + Send {
        async_stream::stream! {
            let mut stream = pin!(input);

            while let Some(res) = stream.next().await {
                match res {
                    Ok(item) => {
                        match (self.f)(item).await {
                            Ok(x) => {
                                let mut x = pin!(x);
                                while let Some(res) = x.next().await {
                                    yield res;
                                }
                            }
                            Err(err) => yield Err(err)
                        }
                    }
                    Err(err) => yield Err(err)
                }
            }
        }
    }
}
