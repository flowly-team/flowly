use flowly::{Context, Service, ServiceExt, flow};
use futures::{FutureExt, Stream, StreamExt, stream};

#[derive(Debug)]
pub enum Error {
    Test,
}

pub struct SvcI32;
impl Service<i32> for SvcI32 {
    type Out = Result<u32, Error>;

    fn handle(&mut self, item: i32, _cx: &Context) -> impl Stream<Item = Self::Out> {
        async move { Ok(item as u32) }.into_stream()
    }
}

pub struct SvcU64;
impl Service<u64> for SvcU64 {
    type Out = Result<u32, Error>;

    fn handle(&mut self, item: u64, _cx: &Context) -> impl Stream<Item = Self::Out> {
        async move { Ok(item as u32) }.into_stream()
    }
}

#[tokio::main]
async fn main() {
    let mut service = flow()
        .flow(SvcI32)
        .flow_map(async |x| x as u64)
        .flow_filter_map(async |x| (x % 2 == 0).then_some(x))
        .flow_inspect(|x| {
            println!("{x}");
        })
        .flow(SvcU64);

    let cx = flowly::Context::new();
    let data: Vec<_> = service
        .handle_stream(stream::iter([0, 1, 2, 3, 4, 5, 6, 7, 8, 9]), &cx)
        .collect()
        .await;

    println!("{data:?}")
}
