use std::marker::PhantomData;

use futures::Stream;

use crate::{Context, Service};

#[inline]
pub fn stub<O>() -> Stub<O> {
    Stub(PhantomData)
}

#[derive(Debug, Clone)]
pub struct Stub<O>(PhantomData<O>);

impl<I, O> Service<I> for Stub<O> {
    type Out = O;

    fn handle(&mut self, _: I, _cx: &Context) -> impl Stream<Item = Self::Out> {
        futures::stream::empty()
    }
}
