use crate::codec::Codec;

use bitflags::bitflags;
use bytes::Bytes;

bitflags! {
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct FrameFlags: u64 {
        const KEY_FRAME        = 0b1 << 0;
        const LAST_FRAME       = 0b1 << 1;
        const LIVE_STREAM      = 0b1 << 2;
        const TIME_SYNCRONIZED = 0b1 << 3;
        const HAS_DTS          = 0b1 << 4;
        const HAS_PARAMS       = 0b1 << 5;
        const HAS_TIMESTAMP    = 0b1 << 6;
    }
}

pub trait Frame {
    fn seq(&self) -> u64;
    fn pts(&self) -> u64;
    fn timestamp(&self) -> Option<u64>;
    fn dts(&self) -> Option<u64>;
    fn codec(&self) -> Codec;
    fn flags(&self) -> FrameFlags;
    fn params(&self) -> impl Iterator<Item = Bytes>;
    fn units(&self) -> impl Iterator<Item = Bytes>;

    fn is_key_frame(&self) -> bool {
        self.flags().contains(FrameFlags::KEY_FRAME)
    }

    fn is_last_frame(&self) -> bool {
        self.flags().contains(FrameFlags::LAST_FRAME)
    }

    fn has_params(&self) -> bool {
        self.flags().contains(FrameFlags::HAS_PARAMS)
    }

    fn has_dts(&self) -> bool {
        self.flags().contains(FrameFlags::HAS_DTS)
    }

    fn has_timestamp(&self) -> bool {
        self.flags().contains(FrameFlags::HAS_TIMESTAMP)
    }
}
