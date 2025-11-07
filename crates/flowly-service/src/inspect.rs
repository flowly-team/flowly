use super::Context;
use super::Service;
use futures::FutureExt;
use futures::Stream;

use std::marker::PhantomData;

#[derive(Debug, Clone)]
pub struct Inspect<I, F> {
    pub(crate) cb: F,
    pub(crate) _m: PhantomData<I>,
}

impl<I, F> Service<I> for Inspect<I, F>
where
    I: Send,
    F: Send + Fn(&I),
{
    type Out = I;

    fn handle(&mut self, input: I, _cx: &Context) -> impl Stream<Item = Self::Out> + Send {
        async move {
            (self.cb)(&input);
            input
        }
        .into_stream()
    }
}
