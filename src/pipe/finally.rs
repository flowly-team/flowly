use std::pin::pin;

use futures::{Stream, StreamExt};

use crate::Service;

pub struct Finally<F> {
    pub(crate) f: F,
}

impl<I, E, C, F> Service<Result<I, E>> for Finally<C>
where
    F: Future<Output = Result<(), E>> + Send,
    C: Send + FnMut() -> F,
    E: Send,
    I: Send,
{
    type Out = Result<I, E>;

    fn handle(
        mut self,
        input: impl Stream<Item = Result<I, E>> + Send,
    ) -> impl Stream<Item = Self::Out> + Send {
        async_stream::stream! {
            let mut stream = pin!(input);

            while let Some(res) = stream.next().await {
                yield res;
            }

            if let Err(err) = (self.f)().await {
                yield Err(err);
            }
        }
    }
}

pub struct Except<F> {
    pub(crate) f: F,
}

impl<I, E, C, F> Service<Result<I, E>> for Except<C>
where
    F: Future<Output = Result<(), E>> + Send,
    C: Send + FnMut(E) -> F,
    E: Send,
    I: Send,
{
    type Out = Result<I, E>;

    fn handle(
        mut self,
        input: impl Stream<Item = Result<I, E>> + Send,
    ) -> impl Stream<Item = Self::Out> + Send {
        async_stream::stream! {
            let mut stream = pin!(input);

            while let Some(res) = stream.next().await {
                match res {
                    Ok(x) => {
                        yield Ok(x)
                    },

                    Err(err) => {
                        if let Err(err) = (self.f)(err).await {
                            yield Err(err)
                        }
                    }
                };
            }
        }
    }
}
