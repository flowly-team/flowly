use futures::{Stream, TryStreamExt};

use crate::Service;

#[derive(Debug, Clone)]
pub struct Stub;

impl<I, E> Service<Result<I, E>> for Stub {
    type Out = Result<(), E>;

    fn handle(
        self,
        input: impl Stream<Item = Result<I, E>> + Send,
    ) -> impl Stream<Item = Self::Out> + Send {
        input.try_filter_map(|_| async move { Ok(None) })
    }
}
