use std::sync::Arc;

use crate::fourcc::Fourcc;

use bitflags::bitflags;

bitflags! {
    #[repr(transparent)]
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct FrameFlags: u32 {
        const KEYFRAME             = 0b1 << 0; // Stream can be split on that frame
        const LAST                 = 0b1 << 1; // Last frame indicator
        const LIVE                 = 0b1 << 2; // Is live stream (IP Camera)
        const TIME_SYNCED          = 0b1 << 3; // Livestream timestamp synchronized with source
        const HAS_PARAMS           = 0b1 << 4; // Has additional parameters
        const HAS_START_TIMESTAMP  = 0b1 << 5; // Livesctream has TimeStamp of the start
        const MULTICHANNEL         = 0b1 << 6; // Stream has more than one channel
        const ENCODED              = 0b1 << 7; // Is encoded stream (h264, h265, etc.)
        const DUMMY                = 0b1 << 8; // Generated, non-real video stream
        const ANNEXB               = 0b1 << 9; // AnnexB prefixed

        const AUDIO_STREAM         = 0b1 << 16;
        const VIDEO_STREAM         = 0b1 << 17;
        const METADATA_STREAM      = 0b1 << 18;
    }
}

pub enum FrameSourceKind {
    File,
    Url,
}

pub trait FrameSource: Sync + Send + std::fmt::Display + Clone + PartialEq {
    fn kind(&self) -> FrameSourceKind;
    fn url(&self) -> &str;
}

impl<T: FrameSource> FrameSource for Arc<T> {
    fn kind(&self) -> FrameSourceKind {
        (**self).kind()
    }

    fn url(&self) -> &str {
        (**self).url()
    }
}

#[derive(Debug, PartialEq)]
pub struct ChunkRef<'a, S: FrameSource, D: AsRef<[u8]>> {
    pub source: &'a S,
    pub data: &'a D,
}

impl<'a, S: FrameSource, D: AsRef<[u8]>> AsRef<[u8]> for ChunkRef<'a, S, D> {
    fn as_ref(&self) -> &[u8] {
        self.data.as_ref()
    }
}

impl<'a, S: FrameSource, D: AsRef<[u8]>> ChunkRef<'a, S, D> {
    pub fn new(source: &'a S, data: &'a D) -> Self {
        Self { source, data }
    }
}

impl<'a, S: FrameSource, D: AsRef<[u8]> + Clone> Clone for ChunkRef<'a, S, D> {
    fn clone(&self) -> Self {
        Self {
            source: self.source,
            data: self.data,
        }
    }
}

impl<'a, S: FrameSource, D: AsRef<[u8]>> std::ops::Deref for ChunkRef<'a, S, D> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.source
    }
}

#[derive(Debug, PartialEq)]
pub struct Chunk<S: FrameSource, D: AsRef<[u8]>> {
    pub source: S,
    pub data: D,
}

impl<S: FrameSource, D: AsRef<[u8]>> AsRef<[u8]> for Chunk<S, D> {
    fn as_ref(&self) -> &[u8] {
        self.data.as_ref()
    }
}

impl<S: FrameSource, D: AsRef<[u8]>> Chunk<S, D> {
    pub fn new(source: S, data: D) -> Self {
        Self { source, data }
    }
}

impl<S: FrameSource, D: AsRef<[u8]> + Clone> Clone for Chunk<S, D> {
    fn clone(&self) -> Self {
        Self {
            source: self.source.clone(),
            data: self.data.clone(),
        }
    }
}

impl<S: FrameSource, D: AsRef<[u8]>> std::ops::Deref for Chunk<S, D> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.source
    }
}
/// Chunk of data in the stream
pub trait DataFrame: Send + Clone {
    type Source: FrameSource;
    type Chunk: Clone + Send + AsRef<[u8]>;

    fn chunks(&self) -> impl Send + Iterator<Item = ChunkRef<Self::Source, Self::Chunk>>;

    fn into_chunks(self) -> impl Send + Iterator<Item = Chunk<Self::Source, Self::Chunk>>;

    /// FrameSource - source info structure
    fn source(&self) -> &Self::Source;
}

/// Frame of the data (related to some timestamp, codec and with some flags)
pub trait Frame: DataFrame {
    /// Timestamp in microseconds when frame should appear from start
    fn timestamp(&self) -> u64;

    /// Fourcc signature of the payload type
    fn codec(&self) -> Fourcc;

    /// Frame Flags (keyframe, last, multitrack, live etc.)    
    fn flags(&self) -> FrameFlags;

    /// Check the flag presence    
    #[inline]
    fn has_flag(&self, flag: FrameFlags) -> bool {
        self.flags().contains(flag)
    }

    /// true if frame is IDR (not depends on other frames)
    #[inline]
    fn is_keyframe(&self) -> bool {
        self.has_flag(FrameFlags::KEYFRAME)
    }

    /// true if frame is compressed
    #[inline]
    fn is_live(&self) -> bool {
        self.has_flag(FrameFlags::LIVE)
    }

    /// true if frame is part of multichannel translation
    #[inline]
    fn is_multichannel(&self) -> bool {
        self.has_flag(FrameFlags::MULTICHANNEL)
    }

    /// true if frame is compressed
    #[inline]
    fn is_encoded(&self) -> bool {
        self.has_flag(FrameFlags::ENCODED)
    }

    /// true if frame is last one in the seq
    #[inline]
    fn is_last(&self) -> bool {
        self.has_flag(FrameFlags::LAST)
    }

    /// frame needs updated decoding params
    #[inline]
    fn has_params(&self) -> bool {
        self.has_flag(FrameFlags::HAS_PARAMS)
    }

    /// live stream has timestamp of starting of the translation
    #[inline]
    fn has_start_timestamp(&self) -> bool {
        self.has_flag(FrameFlags::HAS_START_TIMESTAMP)
    }

    /// Audio Frame
    #[inline]
    fn is_audio(&self) -> bool {
        self.has_flag(FrameFlags::AUDIO_STREAM)
    }

    /// Video Frame
    #[inline]
    fn is_video(&self) -> bool {
        self.has_flag(FrameFlags::VIDEO_STREAM)
    }

    /// Metadata Frame
    #[inline]
    fn is_metadata(&self) -> bool {
        self.has_flag(FrameFlags::METADATA_STREAM)
    }
}

pub trait Live: Frame {
    /// The begging of translation timestamp (unixtimespamp in ms from 01:01:1970)
    fn timestamp(&self) -> u64;
}

pub trait EncodedFrame: Frame {
    type Param: AsRef<[u8]>;

    /// Decoding timestamp in microseconds (monotonic increasing timestamp starting from 0 (usually) at the first frame)
    #[inline]
    fn dts(&self) -> u64 {
        self.timestamp()
    }

    /// Presentation timestamp in microseconds - same as DTS, but with frame presentation offset correction
    fn pts(&self) -> i64;

    /// Iterator by NAL units of the parameters (for h264/h265 - VPS,SPS,PPS)
    fn params(&self) -> impl Iterator<Item = &Self::Param>;
}

pub trait Multichannel: Frame {
    /// Track Number frame belongs to (always 0 for singletrack)
    fn track(&self) -> u32;
}

pub trait VideoFrame: Frame {
    /// Video Dimensions (Width, Height)
    fn dimensions(&self) -> (u16, u16);

    /// Bits per pixel (8, 10, 12)
    fn bit_depth(&self) -> u8;
}
