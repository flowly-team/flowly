use bytes::Bytes;
use futures::{StreamExt, prelude::Stream};

use flowy_core::{Codec, Frame, FrameFlags};
use flowy_service::Service;

pub struct WithFragment<F> {
    pub index: u64,
    pub timestamp: u64,
    pub frame: F,
}

impl<F: Frame> Frame for WithFragment<F> {
    fn seq(&self) -> u64 {
        self.frame.seq()
    }

    fn pts(&self) -> u64 {
        self.frame.pts()
    }

    fn dts(&self) -> Option<u64> {
        self.frame.dts()
    }

    fn timestamp(&self) -> Option<u64> {
        self.frame.timestamp()
    }

    fn codec(&self) -> Codec {
        self.frame.codec()
    }

    fn flags(&self) -> FrameFlags {
        self.frame.flags()
    }

    fn params(&self) -> impl Iterator<Item = Bytes> {
        self.frame.params()
    }

    fn units(&self) -> impl Iterator<Item = Bytes> {
        self.frame.units()
    }
}

pub struct Slicer {
    slice_length: u64,
}

impl Slicer {
    pub fn new(slice_length: u64) -> Self {
        Self {
            slice_length: slice_length * 1000,
        }
    }
}

impl<F: Frame, E: Send + 'static> Service<Result<F, E>> for Slicer {
    type Out = Result<WithFragment<F>, E>;

    fn handle(
        self,
        input: impl Stream<Item = Result<F, E>> + Send,
    ) -> impl Stream<Item = Result<WithFragment<F>, E>> + Send {
        let mut base_ts = None;
        let mut fragment_pts = 0u64;
        let mut counter = 0u64;

        input.map(move |res| match res {
            Ok(frame) => {
                let frame_pts = frame.pts();
                let is_keyframe = frame.is_key_frame();
                let is_last_one = frame.is_last_frame();
                let is_reset_frame = frame.seq() == 0 || base_ts != frame.timestamp();

                let new_fragm = is_reset_frame
                    || is_last_one
                    || (is_keyframe
                        && (frame_pts as i64 - fragment_pts as i64).abs() as u64
                            > self.slice_length);

                if new_fragm {
                    fragment_pts = frame_pts;
                    base_ts = frame.timestamp();
                    counter += 1;
                }

                Ok(WithFragment {
                    index: counter,
                    timestamp: fragment_pts,
                    frame,
                })
            }
            Err(err) => Err(err),
        })
    }
}
