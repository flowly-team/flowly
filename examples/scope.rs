use std::pin::pin;

use flowly::{Context, Service, ServiceExt, flow};
use futures::{Stream, StreamExt, stream};

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

    fn handle(&mut self, item: u64, _cx: &Context) -> impl Stream<Item = Self::Out> {
        stream::iter([
            Ok((item % 3) as _),
            Ok((item % 4) as _),
            Ok((item % 5) as _),
            Ok((item % 6) as _),
        ])
    }
}

#[tokio::main]
async fn main() {
    // let mut service = flow::<Msg>().flow_scope(|x: &Msg| Ok::<_, Error>(x.val), Svc1);
    let mut service = flow::<Msg>().flow_scope_each(|x: &Msg| Ok::<_, Error>(x.val), Svc1);
    let cx = Context::new();
    let mut stream = pin!(service.handle(Msg { val: 12 }, &cx));

    while let Some(res) = stream.next().await {
        println!("{:?}", res);
    }
}
