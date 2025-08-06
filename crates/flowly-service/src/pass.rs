use std::marker::PhantomData;

use flowly_core::Void;
use futures::Stream;

use crate::{Context, Service};

#[derive(Debug, Clone)]
pub struct Pass<I>(pub PhantomData<I>);

impl<I> Service<I> for Pass<I> {
    type Out = Result<I, Void>;

    #[inline]
    fn handle(&mut self, input: I, _cx: &Context) -> impl Stream<Item = Self::Out> {
        futures::stream::once(async move { Ok(input) })
    }
}
