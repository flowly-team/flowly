use futures::{Stream, TryFuture, TryStreamExt};

use crate::Service;

pub struct AndThenFn<C> {
    pub(crate) f: C,
}

impl<I, E, C, F> Service<Result<I, E>> for AndThenFn<C>
where
    F: TryFuture<Error = E> + Send,
    C: FnMut(I) -> F + Send,
{
    type Out = Result<F::Ok, E>;

    fn handle(
        mut self,
        input: impl Stream<Item = Result<I, E>> + Send,
    ) -> impl Stream<Item = Self::Out> + Send {
        input.and_then(move |x| (self.f)(x))
    }
}
