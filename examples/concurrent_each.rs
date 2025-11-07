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

    fn handle(
        &mut self,
        item: i32,
        _cx: &Context,
    ) -> impl futures::Stream<Item = Self::Out> + Send {
        async move {
            println!("start {item}");
            tokio::time::sleep(std::time::Duration::from_millis(item as u64 * 10)).await;
            println!("  end {item}");
            Ok(item as u64)
        }
        .into_stream()
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let mut x = flow() // -
        .flow(Worker)
        .concurrent_each(32);

    let cx = flowly_service::Context::new();
    let vec = x
        .handle_stream(futures::stream::iter((0..100).rev()), &cx)
        .try_flatten_unordered(32)
        .try_collect::<Vec<_>>()
        .await
        .unwrap();

    println!("{:?}", vec);
    // assert_eq!(vec, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
}
