use std::marker::PhantomData;

use futures::Stream;

use crate::{Context, Service};

#[derive(Debug, Clone, Copy)]
pub struct Pass<I, E>(PhantomData<(I, E)>);

impl<I: Send, E: Send> Service<I> for Pass<I, E> {
    type Out = Result<I, E>;

    #[inline]
    fn handle(&mut self, input: I, _cx: &Context) -> impl Stream<Item = Self::Out> + Send {
        futures::stream::once(async move { Ok(input) })
    }
}

#[inline]
pub fn flow<I, E>() -> Pass<I, E> {
    Pass(PhantomData)
}
