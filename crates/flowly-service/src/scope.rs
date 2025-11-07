use std::marker::PhantomData;

use flowly_core::Either;
use futures::{FutureExt, Stream, TryStreamExt};

use crate::Service;

pub fn scope<I, M, E, S, F>(f: F, service: S) -> Scope<I, M, E, S, F> {
    Scope {
        service,
        f,
        _m: PhantomData,
    }
}

#[derive(Clone)]
pub struct Scope<I, M, E, S, F> {
    service: S,
    f: F,
    _m: PhantomData<(I, M, E)>,
}

impl<I, M, E1, O, E, S, F> Service<I> for Scope<I, M, E1, S, F>
where
    S: Service<M, Out = Result<O, E>> + Send,
    F: Send + Fn(&I) -> Result<M, E1>,
    M: Send,
    O: Send,
    I: Send,
    E: Send,
    E1: Send,
{
    type Out = Result<(I, Result<Vec<O>, Either<E, E1>>), E1>;

    fn handle(&mut self, msg: I, cx: &crate::Context) -> impl Stream<Item = Self::Out> + Send {
        async move {
            match (self.f)(&msg) {
                Ok(m) => Ok((
                    msg,
                    self.service
                        .handle(m, cx)
                        .map_err(Either::Left)
                        .try_collect()
                        .await,
                )),
                Err(err) => Ok((msg, Err(Either::Right(err)))),
            }
        }
        .into_stream()
    }
}
