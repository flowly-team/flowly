use crate::codec::Fourcc;

use bitflags::bitflags;

bitflags! {
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct FrameFlags: u64 {
        const KEY_FRAME        = 0b1 << 0;
        const LAST_FRAME       = 0b1 << 1;
        const LIVE_STREAM      = 0b1 << 2;
        const TIME_SYNCRONIZED = 0b1 << 3;
        const HAS_PARAMS       = 0b1 << 4;
        const HAS_TIMESTAMP    = 0b1 << 5;
    }
}

pub trait Frame {
    fn seq(&self) -> u64;
    fn pts(&self) -> i64;
    fn dts(&self) -> u64;
    fn codec(&self) -> Fourcc;
    fn flags(&self) -> FrameFlags;

    fn timestamp(&self) -> Option<u64>;
    fn params(&self) -> impl Iterator<Item = &[u8]>;
    fn units(&self) -> impl Iterator<Item = &[u8]>;

    fn is_key_frame(&self) -> bool {
        self.flags().contains(FrameFlags::KEY_FRAME)
    }

    fn is_last_frame(&self) -> bool {
        self.flags().contains(FrameFlags::LAST_FRAME)
    }

    fn has_params(&self) -> bool {
        self.flags().contains(FrameFlags::HAS_PARAMS)
    }

    fn has_timestamp(&self) -> bool {
        self.flags().contains(FrameFlags::HAS_TIMESTAMP)
    }

    fn units_annexb(&self) -> impl Iterator<Item = &[u8]> {
        self.units().map(|x| [&[0, 0, 0, 1][..], &x[..]]).flatten()
    }
}
