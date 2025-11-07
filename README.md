# Flowly

**Flowly** is a modular, type‑safe, and fully asynchronous Rust library that lets you build robust, data‑driven pipelines for audio, video, and any stream‑based processing.  
It is composed of several lightweight crates that can be used independently or together:

- `flowly-core` – fundamental building blocks (streams, filters, sinks, etc.)  
- `flowly-io` – I/O primitives and adapters for common media formats  
- `flowly-service` – orchestration and lifecycle management of pipeline tasks  
- `flowly-spsc` – single‑producer single‑consumer zero‑allocation channel

All components are designed to work seamlessly with `tokio` and `futures`.

> [!NOTE]  
> Flowly is still in active development. The current stable release is **0.4.6**.

---

## Features

- **Composable** – combine tiny, well‑tested components into complex workflows.
- **Zero‑allocation** – minimal runtime overhead using lock‑free primitives.
- **Async‑first** – built on top of `async-stream`, `futures` and `tokio`.
- **Extensible** – easy to add custom components, adapters, or backends.
- **Cross‑platform** – works on Linux, macOS, Windows, and WASM (with `wasm-bindgen`).

---

## Quick Start

```bash
$ cargo add flowly-core flowly-io flowly-service flowly-spsc
```

```rust
use flowly_core::{pipeline, Component};
use flowly_io::video::VideoSource;
use flowly_service::run;
use tokio::runtime::Runtime;

#[tokio::main]
async fn main() {
    // Build a simple pipeline: source → filter → sink
    let pipeline = pipeline![
        VideoSource::new("input.mp4")?,           // Component::source
        flowly_core::filters::FrameThrottler::new(15), // Component::filter
        flowly_io::video::VideoSink::new("output.mp4")? // Component::sink
    ];

    // Run the pipeline
    run(pipeline).await.expect("Pipeline failed");
}
```

> The example above uses the `pipeline!` macro to wire components together. Each component follows a common trait signature, making it trivial to swap implementations.

---

## Documentation

Full API reference and guides are available on [docs.rs](https://docs.rs/flowly-core/).  
See the *Examples* folder for more comprehensive tutorials:

- [Audio pipeline example](/examples/audio_pipeline.rs)
- [Video transcoding example](/examples/video_transcoding.rs)

---

## Architecture

```
┌───────────────────────────────────────────────────────────────┐
│                       Flowly Core (0.4)                       │
│ ┌─────────────────┐   ┌───────────────────────────────┐      │
│ │  Streams/Forks  │←→│  Filters / Transformations   │      │
│ └─────────────────┘   └───────────────────────────────┘      │
│                           │                                    │
│                           ▼                                    │
└───┬───────────────────────────────────────────────────────────────┘
    │
    ▼
 ┌───────────────────────┐
 │  Flowly Service (0.4)  │  (Orchestration & lifecycle)
 └───────────────────────┘
    ▲
    │
    ▼
 ┌───────────────────────┐
 │    Flowly IO (0.4)     │ (I/O adapters, codecs)
 └───────────────────────┘
    ▲
    │
    ▼
 ┌───────────────────────┐
 │   Flowly SPSC (0.4)    │ (Zero‑allocation channel)
 └───────────────────────┘
```

All crates expose a minimal surface and use `pub use` to re‑export the most important items.  
This structure keeps the core lightweight while still giving you full access to high‑level features.

---

## Getting Started

### Prerequisites

- Rust 1.75+ (stable)
- Cargo
- `tokio` runtime (installed as dependency)

### Installing

```bash
$ cargo add flowly-core flowly-io flowly-service flowly-spsc
```

### Running the examples

```bash
cd examples
cargo run --example audio_pipeline
```

---

## Contribution Guide

We welcome contributions! 

Run the test suite:

```bash
cargo test --workspace
```

---

## License

MIT License – see [LICENSE](LICENSE)

---

## Community

- GitHub Issues: https://github.com/flowly-team/flowly/issues  
- Discuss on [GitHub Discussions](https://github.com/flowly-team/flowly/discussions)

---

## Roadmap

| Feature | Status |
|---------|--------|
| WASM support | ❌ |
| GPU acceleration | ❌ |
| Built‑in AI inference | ❌ |
| Extended audio codecs | ❌ |

---

## Acknowledgements

- The Rust async ecosystem  
- The [tokio](https://tokio.rs/) team  
- The open‑source community that inspires modular design
