use std::marker::PhantomData;

use futures::{Stream, StreamExt};

use crate::{Context, Service, stub::Stub};

pub trait SwitchCaseMatch<T: ?Sized + PartialEq> {
    fn check_match(&self, needle: &T) -> bool;
}

impl<const N: usize, T: ?Sized + PartialEq> SwitchCaseMatch<T> for [&T; N] {
    fn check_match(&self, needle: &T) -> bool {
        self.iter().any(|&x| x == needle)
    }
}

impl<const N: usize, T: ?Sized + PartialEq> SwitchCaseMatch<T> for &[&T; N] {
    fn check_match(&self, needle: &T) -> bool {
        self.iter().any(|&x| x == needle)
    }
}

impl<const N: usize, T: PartialEq> SwitchCaseMatch<T> for [T; N] {
    fn check_match(&self, needle: &T) -> bool {
        self.iter().any(|x| x == needle)
    }
}

impl<const N: usize, T: PartialEq> SwitchCaseMatch<T> for &[T; N] {
    fn check_match(&self, needle: &T) -> bool {
        self.iter().any(|x| x == needle)
    }
}

impl<T: PartialEq> SwitchCaseMatch<T> for T {
    fn check_match(&self, needle: &T) -> bool {
        self == needle
    }
}

impl<T: ?Sized + PartialEq> SwitchCaseMatch<T> for &T {
    fn check_match(&self, needle: &T) -> bool {
        self == &needle
    }
}

impl<T: ?Sized + PartialEq> SwitchCaseMatch<T> for &[&T] {
    fn check_match(&self, needle: &T) -> bool {
        self.iter().any(|&x| x == needle)
    }
}

impl<T: PartialEq> SwitchCaseMatch<T> for &[T] {
    fn check_match(&self, needle: &T) -> bool {
        self.iter().any(|x| x == needle)
    }
}

pub struct Filter<I, F, S> {
    f: F,
    s: S,
    _m: PhantomData<I>,
}

impl<I, F, S> Service<I> for Filter<I, F, S>
where
    S: Service<I>,
    F: Fn(&I) -> bool,
{
    type Out = S::Out;

    fn handle(&mut self, input: I, cx: &Context) -> impl Stream<Item = Self::Out> {
        if (self.f)(&input) {
            self.s.handle(input, cx).left_stream()
        } else {
            futures::stream::empty().right_stream()
        }
    }
}

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

pub enum Case<M, S> {
    Case(M, S),
    Default(S),
    Stub,
}

pub struct Switch<I, M, F, D, S> {
    selector: F,
    case: Case<M, S>,
    m: PhantomData<(I, D)>,
}

impl<I, O, F, D, S> Service<I> for Switch<I, O, F, D, S>
where
    D: PartialEq,
    F: Copy + Fn(&I) -> D,
    S: Service<I, Out = O>,
{
    type Out = O;

    fn handle(&mut self, input: I, cx: &Context) -> impl Stream<Item = Self::Out> {
        async_stream::stream! {}
    }
}

impl<I, O, M, F, D, S> Switch<I, M, F, D, S>
where
    D: PartialEq,
    F: Copy + for<'a> Fn(&'a I) -> D + 'static,
    S: Service<I, Out = O>,
{
    #[inline]
    pub fn case<A, Cs>(
        self,
        variant: A,
        service: Cs,
    ) -> Switch<I, A, F, D, impl Service<I, Out = O>>
    where
        A: SwitchCaseMatch<D>,
        Cs: Service<I, Out = O>,
    {
        Switch {
            selector: self.selector,
            m: PhantomData,
            service: map_if_else(
                move |x| {
                    let d = (self.selector)(x);

                    variant.check_match(&d)
                },
                service,
                self.service,
            ),
            case: Case::Case(variant, service),
        }
    }

    #[inline]
    pub fn default<Cs>(self, service: Cs) -> Switch<I, O, F, D, impl Service<I, Out = O>>
    where
        Cs: Service<I, Out = O>,
    {
        Switch {
            selector: self.selector,
            m: PhantomData,
            case: Case::Default(service),
        }
    }
}

pub fn switch<I, O, F, D>(selector: F) -> Switch<I, O, F, D, Stub<O>>
where
    F: Fn(&I) -> D,
{
    Switch {
        selector,
        m: PhantomData,
        case: Case::Stub,
    }
}
