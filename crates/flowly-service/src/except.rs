use tokio_stream::StreamExt;

use crate::Service;

pub struct Except<S, F> {
    pub(crate) service: S,
    pub(crate) on_err: F,
}

impl<I, R, E, E2, F, S> Service<I> for Except<S, F>
where
    S: Service<I, Out = Result<R, E>>,
    F: Send + Sync + Fn(E) -> Option<E2>,
{
    type Out = Result<R, E2>;

    fn handle(
        &mut self,
        input: I,
        cx: &crate::Context,
    ) -> impl futures::Stream<Item = Self::Out> + Send {
        self.service.handle(input, cx).filter_map(|x| match x {
            Ok(ok) => Some(Ok(ok)),
            Err(err) => (self.on_err)(err).map(Err),
        })
    }
}
