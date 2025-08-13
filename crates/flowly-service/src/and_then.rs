use futures::{FutureExt, Stream};

use crate::{Context, Service};

pub fn and_then<C, F>(f: C) -> AndThenFn<C> {
    AndThenFn { f }
}

pub struct AndThenFn<C> {
    pub(crate) f: C,
}

impl<I: Send, O, E, C, F> Service<Result<I, E>> for AndThenFn<C>
where
    F: Future<Output = Result<O, E>> + Send,
    C: FnMut(I) -> F + Send,
    E: std::marker::Send,
{
    type Out = Result<O, E>;

    fn handle(
        &mut self,
        input: Result<I, E>,
        _cx: &Context,
    ) -> impl Stream<Item = Self::Out> + Send {
        async move {
            match input {
                Ok(ok) => (self.f)(ok).await,
                Err(err) => Err(err),
            }
        }
        .into_stream()
    }
}
