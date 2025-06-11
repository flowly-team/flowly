use crate::fourcc::Fourcc;

use bitflags::bitflags;

bitflags! {
    #[repr(transparent)]
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct FrameFlags: u32 {
        const KEYFRAME         = 0b1 << 0;
        const LAST             = 0b1 << 1;
        const LIVE             = 0b1 << 2;
        const TIME_SYNCED      = 0b1 << 3;
        const HAS_PARAMS       = 0b1 << 4;
        const HAS_TIMESTAMP    = 0b1 << 5;
        const MULTICHANNEL     = 0b1 << 6;
        const COMPRESSED       = 0b1 << 7;

        const AUDIO_STREAM     = 0b1 << 16;
        const VIDEO_STREAM     = 0b1 << 17;
        const METADATA_STREAM  = 0b1 << 18;
    }
}

pub trait Frame {
    /// Decoding timestamp in microseconds (monotonic increasing timestamp starting from 0 (usually) at the first frame)
    fn dts(&self) -> u64;

    /// Presentation timestamp in microseconds - same as DTS, but with frame presentation offset correction
    fn pts(&self) -> i64;

    /// Fourcc signature of the payload type
    fn codec(&self) -> Fourcc;

    /// Frame Flags (keyframe, last, multitrack, live etc.)    
    fn flags(&self) -> FrameFlags;

    /// Track Number frame belongs to (always 0 for singletrack)
    fn track(&self) -> u32;

    /// The begging of translation timestamp (unixtimespamp in ms from 01:01:1970)
    fn timestamp(&self) -> Option<u64>;

    /// Iterator by NAL units of the parameters (for h264/h265 - VPS,SPS,PPS), without annexb prefix
    fn params(&self) -> impl Iterator<Item = &[u8]>;

    /// Iterator by NAL units without annexb prefix
    fn units(&self) -> impl Iterator<Item = &[u8]>;

    /// true if frame is IDR (not depends on other frames)
    fn is_keyframe(&self) -> bool {
        self.flags().contains(FrameFlags::KEYFRAME)
    }

    /// true if frame is part of multichannel translation
    fn is_multichannel(&self) -> bool {
        self.flags().contains(FrameFlags::KEYFRAME)
    }

    /// true if frame is compressed
    fn is_compressed(&self) -> bool {
        self.flags().contains(FrameFlags::COMPRESSED)
    }

    /// true if frame is last one in the seq
    fn is_last(&self) -> bool {
        self.flags().contains(FrameFlags::LAST)
    }

    /// frame needs updated decoding params
    fn has_params(&self) -> bool {
        self.flags().contains(FrameFlags::HAS_PARAMS)
    }

    /// live stream has timestamp of starting of the translation
    fn has_timestamp(&self) -> bool {
        self.flags().contains(FrameFlags::HAS_TIMESTAMP)
    }

    /// Audio Frame
    fn is_audio(&self) -> bool {
        self.flags().contains(FrameFlags::AUDIO_STREAM)
    }

    /// Video Frame
    fn is_video(&self) -> bool {
        self.flags().contains(FrameFlags::VIDEO_STREAM)
    }

    /// Metadata Frame
    fn is_metadata(&self) -> bool {
        self.flags().contains(FrameFlags::VIDEO_STREAM)
    }

    /// Iterator by NAL units of the parameters (for h264/h265 - VPS,SPS,PPS), with annexb prefix
    fn units_annexb(&self) -> impl Iterator<Item = &[u8]> {
        self.units().map(|x| [&[0, 0, 0, 1][..], &x[..]]).flatten()
    }
}
