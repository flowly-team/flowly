use futures::{FutureExt, Stream};

use crate::{Context, Service};

pub fn and_then<C, F>(f: C) -> AndThenFn<C> {
    AndThenFn { f }
}

pub struct AndThenFn<C> {
    pub(crate) f: C,
}

impl<I, O, E, C, F> Service<Result<I, E>> for AndThenFn<C>
where
    F: Future<Output = Result<O, E>>,
    C: FnMut(I) -> F,
{
    type Out = Result<O, E>;

    fn handle(&mut self, input: Result<I, E>, _cx: &Context) -> impl Stream<Item = Self::Out> {
        async move {
            match input {
                Ok(ok) => (self.f)(ok).await,
                Err(err) => Err(err),
            }
        }
        .into_stream()
    }
}
