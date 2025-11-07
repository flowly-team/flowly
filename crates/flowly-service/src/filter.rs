use std::marker::PhantomData;

use futures::{Stream, StreamExt};

use crate::Service;

#[allow(dead_code)]
#[derive(Clone)]
pub struct Filter<I, F, S> {
    f: F,
    s: S,
    _m: PhantomData<I>,
}

impl<I, F, S> Service<I> for Filter<I, F, S>
where
    S: Service<I>,
    S::Out: Send,
    F: Fn(&I) -> bool,
{
    type Out = S::Out;

    fn handle(&mut self, input: I, cx: &crate::Context) -> impl Stream<Item = Self::Out> {
        if (self.f)(&input) {
            self.s.handle(input, cx).left_stream()
        } else {
            futures::stream::empty().right_stream()
        }
    }
}
