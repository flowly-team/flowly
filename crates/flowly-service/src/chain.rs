use futures::Stream;

use crate::Service;

#[derive(Debug, Clone)]
pub struct Chain<S1, S2> {
    pub(crate) service1: S1,
    pub(crate) service2: S2,
}

impl<I, S1: Service<I>, S2: Service<S1::Out>> Service<I> for Chain<S1, S2> {
    type Out = S2::Out;

    fn handle(self, input: impl Stream<Item = I> + Send) -> impl Stream<Item = Self::Out> + Send {
        self.service2.handle(self.service1.handle(input))
    }
}
