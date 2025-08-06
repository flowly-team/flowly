use flowly_io::file::DirReader;
use flowly_service::{Service, ServiceExt, flow};
use futures::TryStreamExt;

#[tokio::main]
async fn main() {
    let mut ppl = flow() // -
        .flow(DirReader::new("*.flv".to_string(), Default::default()));

    let a: [&'static str; 3] = [
        "/home/andrey/demo/av1/",
        "/home/andrey/demo/h264/",
        "/home/andrey/demo/h265/",
    ];

    let cx = flowly_service::Context::new();

    let y = ppl.handle_stream(futures::stream::iter(a), &cx);

    println!("{:#?}", y.try_collect::<Vec<_>>().await);

    ppl.finalize(&cx).await;
}
