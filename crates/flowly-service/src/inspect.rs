use super::Context;
use super::Service;
use futures::FutureExt;
use futures::Stream;

use std::marker::PhantomData;

#[derive(Debug, Clone)]
pub struct Inspect<I, E, F> {
    pub(crate) cb: F,
    pub(crate) _m: PhantomData<(I, E)>,
}

impl<I, E, F> Service<I> for Inspect<I, E, F>
where
    I: Send,
    E: Send,
    F: Send + Fn(&I),
{
    type Out = Result<I, E>;

    fn handle(&mut self, input: I, _cx: &Context) -> impl Stream<Item = Self::Out> + Send {
        async move {
            (self.cb)(&input);
            Ok(input)
        }
        .into_stream()
    }
}
