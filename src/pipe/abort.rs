use futures::{FutureExt, Stream};
use stream_cancel::StreamExt;

use crate::Service;

pub struct AbortFn<F, S> {
    pub(crate) token: F,
    pub(crate) service: S,
}

impl<I, F, S> Service<I> for AbortFn<F, S>
where
    F: Future + Send,
    S: Service<I>,
{
    type Out = S::Out;

    fn handle(self, input: impl Stream<Item = I> + Send) -> impl Stream<Item = Self::Out> + Send {
        self.service
            .handle(input)
            .take_until_if(self.token.map(|_| true))
    }
}
