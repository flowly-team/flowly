use std::pin::pin;

use futures::StreamExt;
use piper::{Service, ServiceExt, pipeline};

#[derive(Debug)]
pub enum Error1 {
    Test,
}
pub struct Service1;

impl<E: Send> Service<Result<i32, E>> for Service1 {
    type Out = Result<u64, Error2<E>>;

    fn handle(
        self,
        input: impl futures::Stream<Item = Result<i32, E>> + Send,
    ) -> impl futures::Stream<Item = Self::Out> + Send {
        async_stream::try_stream! {
            let mut input = pin!(input);
            while let Some(res) = input.next().await {
                let item = res.map_err(Error2::Other)?;
                yield item as u64 ;
            }
        }
    }
}
#[derive(Debug)]
pub enum Error2<E> {
    Other(E),
}
pub struct Service2;

#[tokio::main]
async fn main() {
    let x = pipeline() // -
        .pipe(Service1);

    let y = x.handle(futures::stream::iter([1i32, 2, 3, 4]).map(Ok::<i32, Error1>));

    println!("{:?}", y.collect::<Vec<_>>().await);
}
