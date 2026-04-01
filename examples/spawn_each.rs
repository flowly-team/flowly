use flowly::Context;
use flowly_service::{Service, ServiceExt, flow};
use futures::{FutureExt, TryStreamExt};

#[derive(Debug, Clone, Copy)]
pub enum Error {
    Test,
}

#[derive(Clone)]
pub struct Worker;
impl Service<i32> for Worker {
    type Out = Result<u64, Error>;

    fn handle(&self, item: i32, _cx: &Context) -> impl futures::Stream<Item = Self::Out> + Send {
        async move {
            tokio::time::sleep(std::time::Duration::from_millis(item as u64 * 10)).await;
            Ok(item as u64)
        }
        .into_stream()
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let x = flow() // -
        .flow(Worker)
        .spawn_each();

    let cx = flowly_service::Context::new();
    let vec = x
        .handle_stream(
            futures::stream::iter([10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0]),
            &cx,
        )
        .try_flatten_unordered(16)
        .try_collect::<Vec<_>>()
        .await
        .unwrap();

    assert_eq!(vec, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
}
