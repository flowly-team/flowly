use std::marker::PhantomData;

use futures::{FutureExt, Stream, StreamExt, TryStreamExt};

use crate::{Context, Service};

pub fn map<U, F>(map: F) -> Map<U, F> {
    Map {
        map,
        m: PhantomData,
    }
}

#[derive(Debug, Clone)]
pub struct Map<U, F> {
    pub(crate) map: F,
    pub(crate) m: PhantomData<U>,
}

impl<I, U, H, F> Service<I> for Map<U, F>
where
    F: FnMut(I) -> H + Send,
    H: Future<Output = U> + Send,
{
    type Out = U;

    fn handle(&mut self, input: I, _cx: &Context) -> impl Stream<Item = Self::Out> + Send {
        (self.map)(input).into_stream()
    }
}

pub fn filter_map<U, F>(map: F) -> FilterMap<U, F> {
    FilterMap {
        map,
        m: PhantomData,
    }
}

#[derive(Debug, Clone)]
pub struct FilterMap<U, F> {
    pub(crate) map: F,
    pub(crate) m: PhantomData<U>,
}

impl<I, U, H, F> Service<I> for FilterMap<U, F>
where
    F: FnMut(I) -> H + Send,
    H: Future<Output = Option<U>> + Send,
    U: Send,
{
    type Out = U;

    fn handle(&mut self, input: I, _cx: &Context) -> impl Stream<Item = Self::Out> + Send {
        (self.map)(input).into_stream().filter_map(async |x| x)
    }
}

pub fn try_map<U, E, F>(map: F) -> TryMap<U, E, F> {
    TryMap {
        map,
        m: PhantomData,
    }
}

#[derive(Debug, Clone)]
pub struct TryMap<U, E, F> {
    pub(crate) map: F,
    pub(crate) m: PhantomData<(U, E)>,
}

impl<I, U, E, H, F> Service<I> for TryMap<U, E, F>
where
    F: FnMut(I) -> H + Send,
    H: Future<Output = Result<U, E>> + Send,
{
    type Out = Result<U, E>;

    fn handle(&mut self, input: I, _cx: &Context) -> impl Stream<Item = Self::Out> {
        (self.map)(input).into_stream()
    }
}

pub fn try_filter_map<U, E, F>(map: F) -> TryFilterMap<U, E, F> {
    TryFilterMap {
        map,
        m: PhantomData,
    }
}

#[derive(Debug, Clone)]
pub struct TryFilterMap<U, E, F> {
    pub(crate) map: F,
    pub(crate) m: PhantomData<(U, E)>,
}

impl<I, U, E, H, F> Service<I> for TryFilterMap<U, E, F>
where
    F: FnMut(I) -> H,
    H: Future<Output = Result<Option<U>, E>> + Send,
    U: Send,
{
    type Out = Result<U, E>;

    fn handle(&mut self, input: I, _cx: &Context) -> impl Stream<Item = Self::Out> {
        (self.map)(input)
            .into_stream()
            .try_filter_map(async |x| Ok(x))
    }
}

#[derive(Clone)]
pub struct MapIfElse<I, F, S1, S2> {
    f: F,
    on_true: S1,
    on_false: S2,
    _m: PhantomData<I>,
}

pub fn map_if_else<I, O, F, S1, S2>(f: F, on_true: S1, on_false: S2) -> impl Service<I, Out = O>
where
    F: Fn(&I) -> bool,
    S1: Service<I, Out = O>,
    S2: Service<I, Out = O>,
{
    MapIfElse {
        f,
        on_true,
        on_false,
        _m: PhantomData,
    }
}

impl<I, O, F, S1, S2> Service<I> for MapIfElse<I, F, S1, S2>
where
    S1: Service<I, Out = O>,
    S2: Service<I, Out = O>,
    F: for<'a> Fn(&'a I) -> bool,
{
    type Out = O;

    fn handle(&mut self, input: I, cx: &Context) -> impl Stream<Item = Self::Out> {
        if (self.f)(&input) {
            self.on_true.handle(input, cx).left_stream()
        } else {
            self.on_false.handle(input, cx).right_stream()
        }
    }
}
