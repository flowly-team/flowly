use std::sync::Arc;

use futures::Stream;

use flowly_core::{Codec, Frame};
use flowly_service::Service;

pub struct DecodedFrame {
    data: Arc<Vec<u8>>,
    codec: Codec,
    seq: u64,
    pts: u64,
    timestamp: Option<u64>,
}

pub struct Decoder {
    // parser: Option<Box<dyn StreamParser>>,
    // decoder: Box<dyn StreamDecoder>,
}

impl<F, E> Service<Result<F, E>> for Decoder
where
    E: std::error::Error + Send + Sync + 'static,
    F: Frame + Send,
{
    type Out = Result<F, E>;

    fn handle(
        self,
        input: impl Stream<Item = Result<F, E>> + Send,
    ) -> impl Stream<Item = Self::Out> + Send {
        futures::stream::empty()
    }
}
