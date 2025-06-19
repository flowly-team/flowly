use std::marker::PhantomData;

use futures::Stream;

use crate::Service;

#[derive(Debug, Clone)]
pub struct Pass<I>(pub PhantomData<I>);

impl<I> Service<I> for Pass<I> {
    type Out = I;

    #[inline]
    fn handle(self, input: impl Stream<Item = I>) -> impl Stream<Item = Self::Out> {
        input
    }
}
