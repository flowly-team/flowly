use std::marker::PhantomData;

use flowly_core::Void;
use futures::{FutureExt, Stream, TryStreamExt};

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
    type Out = Result<U, Void>;

    fn handle(&mut self, input: I, _cx: &Context) -> impl Stream<Item = Self::Out> + Send {
        (self.map)(input).map(Ok).into_stream()
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
    type Out = Result<U, Void>;

    fn handle(&mut self, input: I, _cx: &Context) -> impl Stream<Item = Self::Out> + Send {
        (self.map)(input)
            .map(Ok)
            .into_stream()
            .try_filter_map(async |x| Ok(x))
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
