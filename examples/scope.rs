use std::pin::pin;

use flowly::{Context, Service, ServiceExt, flow};
use futures::{FutureExt, Stream, StreamExt};

#[derive(Debug, Clone)]
pub struct Msg {
    val: u64,
}

#[derive(Debug)]
pub enum Error {
    Test,
}

pub struct Svc1;
impl Service<u64> for Svc1 {
    type Out = Result<i32, Error>;

    fn handle(&self, item: u64, _cx: &Context) -> impl Stream<Item = Self::Out> {
        async move { Ok((item % 3) as i32 - 1) }.into_stream()
    }
}

#[tokio::main]
async fn main() {
    // let  service = flow::<Msg, Error>().flow_scope(|x: &Msg| Ok(x.val), Svc1);
    let service = flow::<Msg, Error>().flow_scope_each(|x: &Msg| Ok(x.val), Svc1);
    let cx = Context::new();
    let mut stream = pin!(service.handle(Msg { val: 12 }, &cx));

    while let Some(res) = stream.next().await {
        println!("{:?}", res);
    }
}
