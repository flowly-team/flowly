use std::{
    pin::{Pin, pin},
    task::Poll,
};

use flowly_core::Either;
use futures::{Stream, StreamExt, TryStreamExt};

use crate::Service;

#[derive(Debug, Clone)]
pub struct Maybe<C, S> {
    pub(crate) service: S,
    pub(crate) cond: C,
}

enum Done<T> {
    Item(T),
    Done,
}

pin_project_lite::pin_project! {
struct DoneWrapper<S> {
    #[pin]
    stream: S,
    done_flag: bool
}}

impl<S: Stream> Stream for DoneWrapper<S> {
    type Item = Done<S::Item>;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let this = self.project();

        if *this.done_flag {
            return Poll::Ready(None);
        } else {
            match this.stream.poll_next(cx) {
                Poll::Ready(Some(val)) => Poll::Ready(Some(Done::Item(val))),
                Poll::Ready(None) => {
                    *this.done_flag = true;
                    Poll::Ready(Some(Done::Done))
                }
                Poll::Pending => Poll::Pending,
            }
        }
    }
}

fn done_wrapper<S>(stream: S) -> DoneWrapper<S> {
    DoneWrapper {
        stream,
        done_flag: false,
    }
}

impl<I, E, OE, C, S> Service<Result<I, E>> for Maybe<C, S>
where
    E: Send,
    OE: Send,
    I: Send,
    C: Send + Sync + FnMut(&Result<I, E>) -> bool,
    S: Service<Result<I, E>, Out = Result<I, OE>>,
{
    type Out = Result<I, Either<E, OE>>;

    fn handle(
        mut self,
        input: impl Stream<Item = Result<I, E>> + Send,
    ) -> impl Stream<Item = Self::Out> + Send {
        let (mut tx, rx) = flowly_spsc::channel::<Result<I, E>>(1);
        let rx = self.service.handle(rx);

        async_stream::stream! {
            let mut input = pin!(futures::stream::select(
                done_wrapper(input.map_err(Either::<_, Result<I, Either<E, OE>>>::Left).map(Either::Left)),
                rx.map_err(Either::Right).map(Either::Right).map(Done::Item),
            ));

            while let Some(done) = input.next().await {
                match done {
                    Done::Done => tx.close(),
                    Done::Item(Either::Left(x)) => {
                        let x = x.map_err(Either::into_left);

                        if (self.cond)(&x) {
                            if let Err(_) = tx.send(x).await {
                                log::warn!("cannot flow");
                            }
                        } else {
                            yield x.map_err(Either::Left);
                        }
                    }
                    Done::Item(Either::Right(x)) => yield x,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use futures::StreamExt;

    use crate::{Service, TryServiceExt, flow};

    #[tokio::test]
    async fn test_maybe_flow() {
        let flow = flow::<Result<i32, std::io::Error>>() //-
            .maybe_flow(
                |x| match x {
                    Ok(x) => *x % 2 == 0,
                    Err(_) => false,
                },
                flow().map_ok(|x| x * 100),
            );

        let mut data = flow
            .handle(futures::stream::iter([1, 2, 3, 4, 5, 6, 7, 8, 9]).map(Ok))
            .map(|x| x.unwrap())
            .collect::<Vec<_>>()
            .await;

        data.sort(); // ordering is not guarantered

        assert_eq!(data, vec![1, 3, 5, 7, 9, 200, 400, 600, 800]);
    }
}
