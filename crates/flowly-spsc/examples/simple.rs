use std::time::Instant;

use flowly_spsc::channel;
use futures::executor::block_on;

#[tokio::main]
async fn main() {
    let (mut in_tx, mut in_rx) = channel::<(u64, u64)>(1);
    let (mut out_tx, mut out_rx) = channel::<(u64, u64)>(1);

    std::thread::spawn(move || {
        block_on(async move {
            while let Some((a, b)) = in_rx.recv().await {
                out_tx.send((a + b, a * b)).await.unwrap();
            }
        })
    });

    std::thread::spawn(move || {
        block_on(async move {
            while let Some(res) = out_rx.recv().await {
                nop(res)
            }
        })
    });

    println!("start");
    let time = Instant::now();
    for i in 0..1_000_000 {
        in_tx.send((i, i + 1)).await.unwrap();
    }
    println!("elapsed {}", time.elapsed().as_millis());
}

#[inline(never)]
fn nop(_: (u64, u64)) {}
