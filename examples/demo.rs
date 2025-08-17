use flowly::Context;
use flowly_service::{Service, flow};
use futures::StreamExt;

#[derive(Debug)]
pub enum Error1 {
    Test,
}

pub struct Service1;
impl Service<i32> for Service1 {
    type Out = Result<u64, Error2>;

    fn handle(&mut self, item: i32, _cx: &Context) -> impl futures::Stream<Item = Self::Out> {
        async_stream::try_stream! {
            yield item as u64;
        }
    }
}

pub struct Service2;
impl Service<i32> for Service2 {
    type Out = Result<u64, Error2>;

    fn handle(&mut self, item: i32, _cx: &Context) -> impl futures::Stream<Item = Self::Out> {
        async_stream::try_stream! {
            yield item as u64 + 100;
        }
    }
}
#[derive(Debug)]
pub enum Error2 {}
pub struct Service3;
impl Service<i32> for Service3 {
    type Out = Result<u64, Error2>;

    fn handle(&mut self, item: i32, _cx: &Context) -> impl futures::Stream<Item = Self::Out> {
        async_stream::try_stream! {
            yield item as u64 * 100;
        }
    }
}

#[tokio::main]
async fn main() {
    let mut x = flow() // -
        // .flow(
            // switch::<i32, Result<u64, Error2>, _, _>(|x| x % 3)
            //     .default(Service3)
            //     .case(0, Service1)
            //     .case(1, Service2),
        // )
        ;

    let cx = flowly_service::Context::new();
    let y = x.handle_stream(
        futures::stream::iter([0, 1i32, 2, 3, 4, 5, 6, 7, 8, 9]),
        &cx,
    );

    println!("{:?}", y.collect::<Vec<_>>().await);
}
