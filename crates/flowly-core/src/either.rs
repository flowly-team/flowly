use std::{
    error::Error,
    fmt,
    pin::Pin,
    task::{Context, Poll},
};

use futures::Stream;

#[derive(Debug, Clone, PartialEq)]
pub enum Either<L, R = L> {
    Left(L),
    Right(R),
}

impl<L, R> Either<L, R> {
    #[inline]
    pub fn into_left(self) -> L {
        match self {
            Either::Left(left) => left,
            Either::Right(_) => unreachable!(),
        }
    }

    #[inline]
    pub fn into_right(self) -> R {
        match self {
            Either::Left(_) => unreachable!(),
            Either::Right(right) => right,
        }
    }
}

impl<L: fmt::Display, R: fmt::Display> fmt::Display for Either<L, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Either::Left(l) => write!(f, "{l}"),
            Either::Right(r) => write!(f, "{r}"),
        }
    }
}

impl<L: Error, R: Error> Error for Either<L, R> {}
impl<I, L: Iterator<Item = I>, R: Iterator<Item = I>> Iterator for Either<L, R> {
    type Item = I;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Either::Left(l) => l.next(),
            Either::Right(r) => r.next(),
        }
    }
}

impl<I, L: Future<Output = I>, R: Future<Output = I>> Future for Either<L, R> {
    type Output = I;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match unsafe { self.get_unchecked_mut() } {
            Either::Left(l) => unsafe { Pin::new_unchecked(l) }.poll(cx),
            Either::Right(r) => unsafe { Pin::new_unchecked(r) }.poll(cx),
        }
    }
}

impl<I, L: Stream<Item = I>, R: Stream<Item = I>> Stream for Either<L, R> {
    type Item = I;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match unsafe { self.get_unchecked_mut() } {
            Either::Left(l) => unsafe { Pin::new_unchecked(l) }.poll_next(cx),
            Either::Right(r) => unsafe { Pin::new_unchecked(r) }.poll_next(cx),
        }
    }
}
