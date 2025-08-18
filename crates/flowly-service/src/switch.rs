use std::marker::PhantomData;

use futures::{Stream, StreamExt};

use crate::{Context, Service, stub::Stub};

pub trait SwitchCaseMatch<T: ?Sized + PartialEq> {
    fn check_match(&self, needle: &T) -> bool;
}

impl<const N: usize, T: ?Sized + PartialEq> SwitchCaseMatch<T> for [&T; N] {
    fn check_match(&self, needle: &T) -> bool {
        self.contains(&needle)
    }
}

impl<const N: usize, T: ?Sized + PartialEq> SwitchCaseMatch<T> for &[&T; N] {
    fn check_match(&self, needle: &T) -> bool {
        self.contains(&needle)
    }
}

impl<const N: usize, T: PartialEq> SwitchCaseMatch<T> for [T; N] {
    fn check_match(&self, needle: &T) -> bool {
        self.contains(needle)
    }
}

impl<const N: usize, T: PartialEq> SwitchCaseMatch<T> for &[T; N] {
    fn check_match(&self, needle: &T) -> bool {
        self.contains(needle)
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
        self.contains(&needle)
    }
}

impl<T: PartialEq> SwitchCaseMatch<T> for &[T] {
    fn check_match(&self, needle: &T) -> bool {
        self.contains(needle)
    }
}

pub trait SwitchCase<I>: Service<I> {
    fn try_match(&self, item: &I) -> bool;
}

impl<I: Send, O: Send> SwitchCase<I> for Stub<O> {
    fn try_match(&self, _item: &I) -> bool {
        false
    }
}

pub struct SwitchDefaultCase<I, S, C> {
    case: Option<C>,
    service: S,
    m: PhantomData<I>,
}

impl<I, S, C> Service<I> for SwitchDefaultCase<I, S, C>
where
    S: Service<I>,
    C: SwitchCase<I, Out = S::Out>,
    S::Out: Send,
{
    type Out = S::Out;

    fn handle(&mut self, input: I, cx: &Context) -> impl Stream<Item = Self::Out> + Send {
        if let Some(case) = &mut self.case
            && case.try_match(&input)
        {
            case.handle(input, cx).left_stream()
        } else {
            self.service.handle(input, cx).right_stream()
        }
    }
}

pub struct SwitchMatchCase<I, F, A, S, C> {
    case: Option<C>,
    selector: F,
    service: S,
    variant: A,
    m: PhantomData<I>,
}

impl<I, F, A, S, C> Service<I> for SwitchMatchCase<I, F, A, S, C>
where
    S: Service<I>,
    C: SwitchCase<I, Out = S::Out>,
    S::Out: Send,
{
    type Out = S::Out;

    fn handle(&mut self, input: I, cx: &Context) -> impl Stream<Item = Self::Out> + Send {
        if let Some(case) = &mut self.case
            && case.try_match(&input)
        {
            case.handle(input, cx).left_stream()
        } else {
            self.service.handle(input, cx).right_stream()
        }
    }
}

impl<I, F, D, A, S, C> SwitchCase<I> for SwitchMatchCase<I, F, A, S, C>
where
    S: Service<I>,
    C: SwitchCase<I, Out = S::Out>,
    A: SwitchCaseMatch<D>,
    F: Fn(&I) -> D,
    S::Out: Send,
    D: std::cmp::PartialEq,
{
    fn try_match(&self, item: &I) -> bool {
        if let Some(case) = &self.case
            && case.try_match(item)
        {
            return true;
        }

        let d = (self.selector)(item);

        self.variant.check_match(&d)
    }
}

impl<I, F, D, A, S, C> SwitchMatchCase<I, F, A, S, C>
where
    I: Send,
    S: Service<I>,
    S::Out: Send,
    F: Clone + Fn(&I) -> D,
    C: SwitchCase<I, Out = S::Out>,
    A: SwitchCaseMatch<D>,
    D: std::cmp::PartialEq,
{
    #[inline]
    pub fn case<B, Cs>(
        self,
        variant: B,
        service: Cs,
    ) -> SwitchMatchCase<I, F, B, Cs, impl SwitchCase<I, Out = S::Out>>
    where
        A: SwitchCaseMatch<D>,
        Cs: Service<I, Out = S::Out>,
    {
        SwitchMatchCase {
            selector: self.selector.clone(),
            case: Some(self),
            variant,
            service,
            m: PhantomData,
        }
    }

    #[inline]
    pub fn default<Cs>(
        self,
        service: Cs,
    ) -> SwitchDefaultCase<I, Cs, impl SwitchCase<I, Out = S::Out>>
    where
        Cs: Service<I, Out = S::Out>,
    {
        SwitchDefaultCase {
            case: Some(self),
            service,
            m: PhantomData,
        }
    }
}

pub struct Switch<I, O, F, D> {
    selector: F,
    m: PhantomData<(I, O, D)>,
}

impl<I, O, F, D> Switch<I, O, F, D>
where
    I: Send,
    D: PartialEq,
    O: Send,
    F: Copy + Fn(&I) -> D,
{
    #[inline]
    pub fn case<A, Cs>(
        self,
        variant: A,
        service: Cs,
    ) -> SwitchMatchCase<I, F, A, Cs, impl SwitchCase<I, Out = O>>
    where
        A: SwitchCaseMatch<D>,
        Cs: Service<I, Out = O>,
    {
        SwitchMatchCase {
            case: None::<Stub<O>>,
            selector: self.selector,
            variant,
            service,
            m: PhantomData,
        }
    }

    #[inline]
    pub fn default<Cs>(self, service: Cs) -> SwitchDefaultCase<I, Cs, impl SwitchCase<I, Out = O>>
    where
        Cs: Service<I, Out = O>,
    {
        SwitchDefaultCase {
            case: None::<Stub<O>>,
            service,
            m: PhantomData,
        }
    }
}

pub fn switch<I, O, F, D>(selector: F) -> Switch<I, O, F, D>
where
    F: Fn(&I) -> D,
{
    Switch {
        selector,
        m: PhantomData,
    }
}
