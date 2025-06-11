#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct Fourcc([u8; 4]);

impl Fourcc {
    pub const AUDIO_AC3: Fourcc = Fourcc(*b"ac-3");
    pub const AUDIO_EC3: Fourcc = Fourcc(*b"ec-3");
    pub const AUDIO_OPUS: Fourcc = Fourcc(*b"Opus");
    pub const AUDIO_MP3: Fourcc = Fourcc(*b".mp3");
    pub const AUDIO_FLAC: Fourcc = Fourcc(*b"fLaC");
    pub const AUDIO_AAC: Fourcc = Fourcc(*b"mp4a");

    pub const VIDEO_VP8: Fourcc = Fourcc(*b"vp08");
    pub const VIDEO_VP9: Fourcc = Fourcc(*b"vp09");
    pub const VIDEO_AV1: Fourcc = Fourcc(*b"av01");
    pub const VIDEO_AVC: Fourcc = Fourcc(*b"avc1");
    pub const VIDEO_HEVC: Fourcc = Fourcc(*b"hvc1");

    pub const fn from_static(s: &str) -> Self {
        assert!(s.len() == 4);

        let bs = s.as_bytes();
        Self([bs[3], bs[2], bs[1], bs[0]])
    }
}

impl From<&str> for Fourcc {
    fn from(s: &str) -> Self {
        Self::from_static(s)
    }
}

impl From<u32> for Fourcc {
    fn from(fourcc: u32) -> Self {
        Self(fourcc.to_be_bytes())
    }
}

impl From<Fourcc> for u32 {
    fn from(fourcc: Fourcc) -> Self {
        u32::from_be_bytes(fourcc.0)
    }
}

impl From<&[u8; 4]> for Fourcc {
    fn from(n: &[u8; 4]) -> Self {
        Self(*n)
    }
}

impl From<Fourcc> for [u8; 4] {
    fn from(n: Fourcc) -> Self {
        n.0
    }
}

impl std::fmt::Display for Fourcc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c: [u8; 4] = (*self).into();

        f.write_fmt(format_args!(
            "{}{}{}{}",
            c[0] as char, c[1] as char, c[2] as char, c[3] as char
        ))
    }
}
