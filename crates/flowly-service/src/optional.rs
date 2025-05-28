use futures::{Stream, StreamExt};

use crate::Service;

#[derive(Debug, Clone)]
pub struct Optional<S> {
    pub(crate) service: Option<S>,
}

impl<I, S: Service<I, Out = I>> Service<I> for Optional<S> {
    type Out = I;

    fn handle(self, input: impl Stream<Item = I> + Send) -> impl Stream<Item = Self::Out> + Send {
        if let Some(srv) = self.service {
            srv.handle(input).left_stream()
        } else {
            input.right_stream()
        }
    }
}
