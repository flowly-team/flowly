use flowly_io::file::DirReader;
use flowly_service::{Service, ServiceExt, pipeline};
use futures::TryStreamExt;

#[tokio::main]
async fn main() {
    let ppl = pipeline() // -
        .pipe(DirReader::new("*.mp4".to_string(), Default::default()));

    let a: [Result<&'static str, std::io::Error>; 3] = [
        Ok("/home/andrey/demo/av1/"),
        Ok("/home/andrey/demo/h264/"),
        Ok("/home/andrey/demo/h265/"),
    ];

    let y = ppl.handle(futures::stream::iter(a));

    println!("{:#?}", y.try_collect::<Vec<_>>().await);
}
