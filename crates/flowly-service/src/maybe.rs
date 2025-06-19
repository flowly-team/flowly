use std::{
    pin::{Pin, pin},
    task::Poll,
};

use futures::{Stream, StreamExt};

use crate::Service;

#[derive(Debug, Clone)]
pub struct Maybe<C, S> {
    pub(crate) service: S,
    pub(crate) cond: C,
}

enum Either<T> {
    Left(T),
    Right(T),
    Done,
}

pin_project_lite::pin_project! {
struct DoneWrapper<S> {
    #[pin]
    stream: S,
    done_flag: bool
}}

impl<S: Stream> Stream for DoneWrapper<S> {
    type Item = Either<S::Item>;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let this = self.project();

        if *this.done_flag {
            return Poll::Ready(None);
        } else {
            match this.stream.poll_next(cx) {
                Poll::Ready(Some(val)) => Poll::Ready(Some(Either::Left(val))),
                Poll::Ready(None) => {
                    *this.done_flag = true;
                    Poll::Ready(Some(Either::Done))
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

impl<I, C, S> Service<I> for Maybe<C, S>
where
    I: Send + std::fmt::Debug,
    C: Send + Sync + Fn(&I) -> bool,
    S: Service<I, Out = I>,
{
    type Out = I;

    fn handle(self, input: impl Stream<Item = I> + Send) -> impl Stream<Item = Self::Out> + Send {
        let (mut tx, rx) = flowly_spsc::channel::<I>(1);
        let rx = self.service.handle(rx);

        async_stream::stream! {
            let mut input = pin!(futures::stream::select(done_wrapper(input), rx.map(Either::Right)));

            while let Some(either) = input.next().await {
                match either {
                    Either::Done => tx.close(),
                    Either::Left(x) => {
                        if (self.cond)(&x) {
                            if let Err(_) = tx.send(x).await {
                                log::warn!("cannot flow");
                            }
                        } else {
                            yield x;
                        }
                    },
                    Either::Right(x) => yield x,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use futures::StreamExt;

    use crate::{Service, ServiceExt, flow};

    #[tokio::test]
    async fn test_maybe_flow() {
        let flow = flow().maybe_flow(|x| x % 2 == 0, flow().map(|x| x * 100));

        let mut data = flow
            .handle(futures::stream::iter([1, 2, 3, 4, 5, 6, 7, 8, 9]))
            .collect::<Vec<_>>()
            .await;

        data.sort(); // ordering is not guarantered

        assert_eq!(data, vec![1, 3, 5, 7, 9, 200, 400, 600, 800]);
    }
}
