#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Codec(u32);
impl Codec {
    // Video
    pub const H264: Codec = Codec::from_static("H264");
    pub const H265: Codec = Codec::from_static("H265");
    pub const AV1: Codec = Codec::from_static("AV1 ");
    pub const MJPEG: Codec = Codec::from_static("MJPG");
    pub const JPEG: Codec = Codec::from_static("JPEG");

    // Audio
    pub const PCM: Codec = Codec::from_static("PCM ");
    pub const PCM_ALAW: Codec = Codec::from_static("PCMa");
    pub const PCM_MULAW: Codec = Codec::from_static("PCMu");
    pub const OPUS: Codec = Codec::from_static("OPUS");
    pub const AAC: Codec = Codec::from_static("AAC ");

    // Unk
    pub const UNK: Codec = Codec::from_static("\0\0\0\0");

    pub fn as_str(&self) -> &str {
        let bytes: &[u8; 4] = unsafe { std::mem::transmute(&self.0) };
        unsafe { str::from_utf8_unchecked(bytes) }
    }

    pub const fn from_static(s: &'static str) -> Codec {
        assert!(s.len() == 4);

        let bs = s.as_bytes();
        Codec(u32::from_be_bytes([bs[3], bs[2], bs[1], bs[0]]))
    }
}

impl std::fmt::Display for Codec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
impl std::fmt::Debug for Codec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Codec({})", self.as_str())
    }
}
