use std::marker::PhantomData;

use futures::{Stream, StreamExt, TryStreamExt};

use crate::Service;

#[derive(Debug, Clone)]
pub struct MapEachFn<U, F> {
    pub(crate) map: F,
    pub(crate) m: PhantomData<U>,
}

impl<I, U, F> Service<I> for MapEachFn<U, F>
where
    F: Send + FnMut(I) -> U,
{
    type Out = U;

    fn handle(self, input: impl Stream<Item = I> + Send) -> impl Stream<Item = Self::Out> + Send {
        input.map(self.map)
    }
}

pub struct MapOk<U, F, E> {
    pub(crate) map: F,
    pub(crate) m: PhantomData<(U, E)>,
}

impl<I, U, F, E> Service<Result<I, E>> for MapOk<U, F, E>
where
    F: Send + FnMut(I) -> U,
{
    type Out = Result<U, E>;

    fn handle(
        self,
        input: impl Stream<Item = Result<I, E>> + Send,
    ) -> impl Stream<Item = Self::Out> + Send {
        input.map_ok(self.map)
    }
}
