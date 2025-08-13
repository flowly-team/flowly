mod and_then;
mod map;
mod pass;
// mod spawn;
mod stub;
// mod switch;
// mod maybe;
// mod optional;
// mod finally;
// mod flatten;

pub use and_then::and_then;
use flowly_core::{Either, Void};
pub use map::{filter_map, map, try_filter_map, try_map};
pub use stub::stub;
// pub use switch::{map_if_else, switch};

use std::{marker::PhantomData, pin::pin};

use futures::{Stream, StreamExt};

#[inline]
pub fn flow<I>() -> pass::Pass<I> {
    pass::Pass(PhantomData)
}

pub struct Context {
    pub abort_tx: tokio::sync::watch::Sender<bool>,
    pub abort: tokio::sync::watch::Receiver<bool>,
}

impl Context {
    pub fn new() -> Self {
        let (abort_tx, abort) = tokio::sync::watch::channel(false);
        Self { abort, abort_tx }
    }
}

pub trait Service<In> {
    type Out;

    fn handle(&mut self, input: In, cx: &Context) -> impl Stream<Item = Self::Out>;

    fn handle_stream(
        &mut self,
        input: impl Stream<Item = In>,
        cx: &Context,
    ) -> impl Stream<Item = Self::Out> {
        async_stream::stream! {
            let mut input = pin!(input);

            while let Some(item) = input.next().await {
                let mut s = pin!(self.handle(item, cx));

                while let Some(out) = s.next().await {
                    yield out;
                }
            }
        }
    }

    fn finalize(&mut self, _cx: &Context) -> impl Future<Output = ()>
    where
        Self: Sized,
    {
        async move {}
    }
}

impl<I, O1, E1, O2, E2, S1, S2> Service<I> for (S1, S2)
where
    S1: Service<I, Out = Result<O1, E1>>,
    S2: Service<O1, Out = Result<O2, E2>>,
{
    type Out = Result<O2, Either<E1, E2>>;

    fn handle(&mut self, msg: I, cx: &Context) -> impl Stream<Item = Self::Out> {
        async_stream::stream! {
            let mut s1 = pin!(self.0.handle(msg, cx));

            while let Some(res) = s1.next().await {
                match res {
                    Ok(ok) => {
                        let mut s2 = pin!(self.1.handle(ok, cx));

                        while let Some(i2) = s2.next().await {
                            yield i2.map_err(Either::Right);
                        }
                    },
                    Err(err) => yield Err(Either::Left(err)),
                }
            }
        }
    }
}

pub trait ServiceExt<I>: Service<I> {
    #[inline]
    fn flow<O1, O2, E1, E2, U>(
        self,
        service: U,
    ) -> impl Service<I, Out = Result<O2, Either<E1, E2>>>
    where
        Self: Sized,
        Self: Service<I, Out = Result<O1, E1>>,
        U: Service<O1, Out = Result<O2, E2>>,
    {
        (self, service)
    }

    // #[inline]
    // fn spawn(self) -> spawn::Spawn<Self>
    // where
    //     Self: Sized,
    // {
    //     spawn::Spawn { service: self }
    // }

    #[inline]
    fn flow_map<O1, O2, E1, F: AsyncFnMut(O1) -> O2>(
        self,
        f: F,
    ) -> impl Service<I, Out = Result<O2, Either<E1, Void>>>
    where
        Self: Sized + Service<I, Out = Result<O1, E1>>,
    {
        (self, map::map::<O2, _>(f))
    }

    #[inline]
    fn flow_filter_map<O1, O2, E1, F: AsyncFnMut(O1) -> Option<O2>>(
        self,
        f: F,
    ) -> impl Service<I, Out = Result<O2, Either<E1, Void>>>
    where
        Self: Sized + Service<I, Out = Result<O1, E1>>,
    {
        (self, map::filter_map::<O2, _>(f))
    }
}

impl<I, T: Service<I>> ServiceExt<I> for T {}

impl<I, E, S: Service<I, Out = Result<I, E>>> Service<I> for Option<S> {
    type Out = Result<I, E>;

    fn handle(&mut self, input: I, cx: &Context) -> impl Stream<Item = Self::Out> {
        if let Some(srv) = self {
            srv.handle(input, cx).left_stream()
        } else {
            futures::stream::once(async move { Ok(input) }).right_stream()
        }
    }
}
