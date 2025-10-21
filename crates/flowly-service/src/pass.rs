use std::marker::PhantomData;

use flowly_core::Void;
use futures::Stream;

use crate::{Context, Service};

#[derive(Debug, Clone, Copy)]
pub struct Pass<I>(PhantomData<I>);

impl<I: Send> Service<I> for Pass<I> {
    type Out = Result<I, Void>;

    #[inline]
    fn handle(&mut self, input: I, _cx: &Context) -> impl Stream<Item = Self::Out> + Send {
        futures::stream::once(async move { Ok(input) })
    }
}

#[inline]
pub fn flow<I>() -> Pass<I> {
    Pass(PhantomData)
}
