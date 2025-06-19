use std::{marker::PhantomData, pin::pin};

use flowly_spsc::Sender;
use futures::{Stream, StreamExt, TryFutureExt};
use tokio::sync::mpsc;

use crate::Service;

pub trait SwitchCase<T> {
    fn test(&self, item: &T) -> bool;
    fn apply(&mut self, item: T) -> impl Future<Output = Result<(), T>> + Send;
}

pub trait SwitchCaseMatch<T: ?Sized + PartialEq> {
    fn check_match(&self, needle: &T) -> bool;
}

// impl<const N: usize, T: ?Sized + PartialEq> SwitchCaseMatch<T> for [&T; N] {
//     fn check_match(&self, needle: &T) -> bool {
//         self.iter().any(|&x| x == needle)
//     }
// }

// impl<const N: usize, T: ?Sized + PartialEq> SwitchCaseMatch<T> for &[&T; N] {
//     fn check_match(&self, needle: &T) -> bool {
//         self.iter().any(|&x| x == needle)
//     }
// }

// impl<const N: usize, T: PartialEq> SwitchCaseMatch<T> for [T; N] {
//     fn check_match(&self, needle: &T) -> bool {
//         self.iter().any(|x| x == needle)
//     }
// }

// impl<const N: usize, T: PartialEq> SwitchCaseMatch<T> for &[T; N] {
//     fn check_match(&self, needle: &T) -> bool {
//         self.iter().any(|x| x == needle)
//     }
// }

// impl<T: PartialEq> SwitchCaseMatch<T> for T {
//     fn check_match(&self, needle: &T) -> bool {
//         self == needle
//     }
// }

// impl<T: ?Sized + PartialEq> SwitchCaseMatch<T> for &T {
//     fn check_match(&self, needle: &T) -> bool {
//         self == &needle
//     }
// }

// impl<T: ?Sized + PartialEq> SwitchCaseMatch<T> for &[&T] {
//     fn check_match(&self, needle: &T) -> bool {
//         self.iter().any(|&x| x == needle)
//     }
// }

// impl<T: PartialEq> SwitchCaseMatch<T> for &[T] {
//     fn check_match(&self, needle: &T) -> bool {
//         self.iter().any(|x| x == needle)
//     }
// }

pub struct ComposeCase<C1, C2> {
    case1: C1,
    case2: C2,
}

impl<T, C1, C2> SwitchCase<T> for ComposeCase<C1, C2>
where
    C1: SwitchCase<T>,
    C2: SwitchCase<T>,
    T: Send,
    Self: Send,
{
    #[inline]
    fn test(&self, case: &T) -> bool {
        self.case1.test(case) || self.case2.test(case)
    }

    async fn apply(&mut self, item: T) -> Result<(), T> {
        if self.case1.test(&item) {
            return self.case1.apply(item).await;
        }

        self.case2.apply(item).await
    }
}

pub struct DefaultCase<I> {
    sender: Sender<I>,
}

impl<I> SwitchCase<I> for DefaultCase<I>
where
    I: Send + Sync,
{
    fn test(&self, _case: &I) -> bool {
        true
    }

    fn apply(&mut self, item: I) -> impl Future<Output = Result<(), I>> + Send {
        self.sender.send(item).map_err(|x| x.val)
    }
}

pub struct MatchCase<I, S, M, Cs> {
    matcher: M,
    selector: S,
    sender: mpsc::Sender<I>,
    service: Cs,
}

impl<I, U, S, M, Cs> SwitchCase<I> for MatchCase<I, S, M, Cs>
where
    M: SwitchCaseMatch<U>,
    S: for<'a> Fn(&'a I) -> &'a U,
    I: Send + Sync + 'static,
    U: ?Sized + PartialEq,
    Self: Send,
{
    fn test(&self, case: &I) -> bool {
        self.matcher.check_match((self.selector)(case))
    }

    async fn apply(&mut self, item: I) -> Result<(), I> {
        if self.test(&item) {
            self.sender.send(item).await.map_err(|x| x.0)
        } else {
            Err(item)
        }
    }
}

pub struct Switch<I, O, F, C> {
    selector: F,
    case: C,
    sender: mpsc::Sender<O>,
    receiver: mpsc::Receiver<O>,
    m: PhantomData<I>,
}

impl<I, O, F, C> Service<I> for Switch<I, O, F, C>
where
    I: Send,
    O: Send,
    C: Send + SwitchCase<I> + 'static,
{
    type Out = O;

    fn handle(
        mut self,
        input: impl Stream<Item = I> + Send,
    ) -> impl Stream<Item = Self::Out> + Send {
        // async_stream::stream! {
        //     let mut input = pin!(input);
        //     let (tx, rx)

        //     while let Some(item) = input.next().await {
        //         if let Err(item) = self.case.apply(item).await {
        //             //
        //         }
        //     }
        // }

        // tokio::spawn(async move {
        //     let mut pinned = pin!(input);

        //     while let Some(res) = pinned.next().await {
        //         if let Err(_item) = self.case.apply(res).await {
        //             // log::warn!("unhandled item!");
        //         }
        //     }
        // });

        futures::stream::poll_fn(move |cx| self.receiver.poll_recv(cx))
    }
}

impl<I, O, U, F, C> Switch<I, O, F, C>
where
    U: ?Sized + PartialEq,
    C: Send + SwitchCase<I> + 'static,
    F: Clone + for<'a> Fn(&'a I) -> &'a U,
    I: Sync + Send + 'static,
    O: Sync + Send + 'static,
{
    #[allow(clippy::type_complexity)]
    #[inline]
    pub fn case<M, Cs>(
        self,
        matcher: M,
        service: Cs,
    ) -> Switch<I, O, F, ComposeCase<C, MatchCase<I, F, M>>>
    where
        M: SwitchCaseMatch<U>,
        Cs: Send + Service<I>,
    {
        Switch {
            case: ComposeCase {
                case1: self.case,
                case2: MatchCase {
                    matcher,
                    service,
                    selector: self.selector.clone(),
                    sender: self.sender.clone(),
                },
            },
            selector: self.selector,
            m: PhantomData,
            sender: self.sender,
            receiver: self.receiver,
        }
    }

    #[inline]
    pub fn default<Cs>(self, service: Cs) -> Switch<I, O, F, ComposeCase<C, DefaultCase<I>>> {
        let (sender, receiver) = flowly_spsc::channel(1);

        Switch {
            case: ComposeCase {
                case1: self.case,
                case2: DefaultCase { sender },
            },
            selector: self.selector,
            m: PhantomData,
            stream: futures::stream::select(service.handle(receiver.map(Ok)), self.stream),
        }
    }
}

pub fn switch<E, Q, F, U, R>(
    selector: F,
) -> Switch<F, VoidCase, R, E, impl Stream<Item = Result<Q, E>>>
where
    U: ?Sized + PartialEq,
    F: Clone + for<'a> Fn(&'a R) -> &'a U,
    R: Send + Sync + 'static,
    E: Send + 'static,
    Q: Send + 'static,
{
    Switch {
        selector,
        case: VoidCase,
        m: PhantomData,
        stream: futures::stream::empty(),
    }
}
