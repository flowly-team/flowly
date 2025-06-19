mod abort;
mod and_then;
mod chain;
mod finally;
mod flatten;
mod map;
mod maybe;
mod optional;
mod pass;
mod spawn;
mod stub;

use std::marker::PhantomData;

use flowly_core::Either;
use futures::{Stream, StreamExt, TryFuture};

pub trait Service<In> {
    type Out;

    fn handle(self, input: impl Stream<Item = In> + Send) -> impl Stream<Item = Self::Out> + Send;
}

pub trait ServiceExt<I>: Service<I> {
    #[inline]
    fn flow<U>(self, service: U) -> impl Service<I, Out = U::Out>
    where
        Self: Sized,
        U: Service<Self::Out>,
    {
        chain::Chain {
            service1: self,
            service2: service,
        }
    }

    #[inline]
    fn spawn(self, buffer: usize) -> spawn::Spawn<Self>
    where
        Self: Sized,
    {
        spawn::Spawn {
            service: self,
            buffer,
        }
    }

    #[inline]
    fn map<U, F>(self, map: F) -> impl Service<I, Out = U>
    where
        Self: Sized,
        F: Send + FnMut(Self::Out) -> U,
    {
        self.flow(map::MapEachFn {
            map,
            m: PhantomData,
        })
    }

    #[inline]
    fn abort_token(self, token: impl futures::Future + Send) -> impl Service<I, Out = Self::Out>
    where
        Self: Sized,
    {
        abort::AbortFn {
            token,
            service: self,
        }
    }
}

impl<I, T: Service<I>> ServiceExt<I> for T {}

pub trait TryService<I, E>: Service<Result<I, E>, Out = Result<Self::Ok, Self::Error>> {
    type Ok;
    type Error;

    fn try_handle(
        self,
        input: impl Stream<Item = Result<I, E>> + Send,
    ) -> impl Stream<Item = Result<Self::Ok, Self::Error>> + Send;
}

impl<S, I, IE, O, OE> TryService<I, IE> for S
where
    S: Service<Result<I, IE>, Out = Result<O, OE>>,
{
    type Ok = O;
    type Error = OE;

    fn try_handle(
        self,
        input: impl Stream<Item = Result<I, IE>> + Send,
    ) -> impl Stream<Item = Result<Self::Ok, Self::Error>> + Send {
        self.handle(input)
    }
}

pub trait TryServiceExt<I, E>: TryService<I, E> {
    #[inline]
    fn maybe_flow<OE, C, S>(
        self,
        cond: C,
        service: S,
    ) -> impl Service<Result<I, E>, Out = Result<Self::Ok, Either<Self::Error, OE>>>
    where
        E: Send,
        OE: Send,
        Self::Ok: Send,
        Self::Error: Send,
        Self: Sized,
        I: Send + std::fmt::Debug,
        C: Send + Sync + Fn(&Result<Self::Ok, Self::Error>) -> bool,
        S: Service<Result<Self::Ok, Self::Error>, Out = Result<Self::Ok, OE>>,
    {
        self.flow(maybe::Maybe { cond, service })
    }

    #[inline]
    fn map_ok<C: FnMut(Self::Ok) -> U + Send, U>(
        self,
        map: C,
    ) -> impl Service<Result<I, E>, Out = Result<U, Self::Error>>
    where
        Self: Sized,
    {
        self.flow(map::MapOk {
            map,
            m: PhantomData,
        })
    }

    #[inline]
    fn and_then<C: FnMut(Self::Ok) -> F + Send, F>(
        self,
        f: C,
    ) -> impl Service<Result<I, E>, Out = Result<F::Ok, Self::Error>>
    where
        Self: Sized,
        F: TryFuture<Error = Self::Error> + Send,
    {
        self.flow(and_then::AndThenFn { f })
    }

    #[inline]
    fn finally<C, F>(self, f: C) -> impl Service<Result<I, E>, Out = Self::Out>
    where
        Self: Sized,
        Self::Ok: Send,
        Self::Error: Send,
        F: Future<Output = Result<(), Self::Error>> + Send,
        C: Send + FnMut() -> F,
    {
        self.flow(finally::Finally { f })
    }

    #[inline]
    fn except<C, F>(self, f: C) -> impl Service<Result<I, E>, Out = Self::Out>
    where
        Self: Sized,
        Self::Ok: Send,
        Self::Error: Send,
        F: Future<Output = Result<(), Self::Error>> + Send,
        C: Send + FnMut(Self::Error) -> F,
    {
        self.flow(finally::Except { f })
    }

    #[inline]
    fn stub(self) -> impl Service<Result<I, E>, Out = Result<(), Self::Error>>
    where
        Self: Sized,
        Self::Ok: Send,
        Self::Error: Send,
    {
        self.flow(stub::Stub)
    }

    fn try_flatten_map<C, O, F, S>(
        self,
        f: C,
    ) -> impl Service<Result<I, E>, Out = Result<O, Self::Error>>
    where
        Self: Sized,
        Self::Ok: Send,
        Self::Error: Send,
        O: Send,
        E: Send,
        I: Send,
        F: Future<Output = Result<S, Self::Error>> + Send,
        C: FnMut(Self::Ok) -> F + Send,
        S: Stream<Item = Result<O, Self::Error>> + Send,
    {
        self.flow(flatten::TryFlattenMap { f })
    }
}

impl<I, E, T: TryService<I, E>> TryServiceExt<I, E> for T {}

impl<I, S: Service<I, Out = I>> Service<I> for Option<S> {
    type Out = I;

    fn handle(self, input: impl Stream<Item = I> + Send) -> impl Stream<Item = Self::Out> + Send {
        if let Some(srv) = self {
            srv.handle(input).left_stream()
        } else {
            input.right_stream()
        }
    }
}

pub fn flow<I>() -> pass::Pass<I> {
    pass::Pass(PhantomData)
}
