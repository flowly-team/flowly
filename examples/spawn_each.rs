use flowly::Context;
use flowly_service::{Service, ServiceExt, flow};
use futures::StreamExt;
use futures::TryStreamExt;
use tokio::time::sleep;

#[derive(Debug, Clone, Copy)]
pub enum Error {
    Test,
}

#[derive(Clone)]
pub struct Worker;
impl Service<i32> for Worker {
    type Out = Result<u64, Error>;

    fn handle(
        &mut self,
        item: i32,
        _cx: &Context,
    ) -> impl futures::Stream<Item = Self::Out> + Send {
        async_stream::try_stream! {
            yield item as u64;
            sleep(std::time::Duration::from_millis(100)).await;
            yield item as u64 + 1;
            sleep(std::time::Duration::from_millis(100)).await;
            yield item as u64 + 2;
            sleep(std::time::Duration::from_millis(100)).await;
            yield item as u64 + 3;
            sleep(std::time::Duration::from_millis(100)).await;
        }
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let mut x = flow() // -
        .flow(Worker)
        .spawn_each();

    let cx = flowly_service::Context::new();
    let y = x
        .handle_stream(
            futures::stream::iter([10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0]),
            &cx,
        )
        .try_flatten_unordered(16);

    y.for_each_concurrent(128, async |x| {
        println!("{} ", x.unwrap());
    })
    .await;

    println!();
}
