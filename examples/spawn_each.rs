use flowly::Context;
use flowly_service::{Service, ServiceExt, flow};
use futures::{StreamExt, TryStreamExt};

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
        futures::stream::iter([
            item as u64,
            item as u64 + 1,
            item as u64 + 2,
            item as u64 + 3,
        ])
        .map(Ok)
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let mut x = flow() // -
        .flow(Worker)
        .spawn_each();

    let cx = flowly_service::Context::new();
    let y = x.handle_stream(
        futures::stream::iter([0i32, 10, 20, 30, 40, 50, 60, 70, 80, 90]),
        &cx,
    );
}
