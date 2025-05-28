use futures::Stream;

use crate::Service;

#[derive(Debug, Clone)]
pub struct Pass;

impl<I> Service<I> for Pass {
    type Out = I;

    #[inline]
    fn handle(self, input: impl Stream<Item = I>) -> impl Stream<Item = Self::Out> {
        input
    }
}
