#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct Fourcc([u8; 4]);

impl Fourcc {
    ///
    /// Audio Codecs
    ///

    /// Dolby AC-3    
    pub const AUDIO_AC3: Fourcc = Fourcc(*b"ac-3");

    /// Dolby Digital Plus (E-AC-3)
    pub const AUDIO_EC3: Fourcc = Fourcc(*b"ec-3");

    /// Opus Audio Codec
    pub const AUDIO_OPUS: Fourcc = Fourcc(*b"Opus");

    /// Mp3 Audio Codec
    pub const AUDIO_MP3: Fourcc = Fourcc(*b".mp3");

    /// FLAC (Free Lossless Audio Codec)
    pub const AUDIO_FLAC: Fourcc = Fourcc(*b"fLaC");

    /// ACC (Advanced Audio Coding)
    pub const AUDIO_AAC: Fourcc = Fourcc(*b"mp4a");

    /// PCM codec series
    pub const AUDIO_PCM: Fourcc = Fourcc(*b"PCM ");
    pub const AUDIO_PCM_ALAW: Fourcc = Fourcc(*b"ALAW");
    pub const AUDIO_PCM_ULAW: Fourcc = Fourcc(*b"ULAW");

    ///
    /// Video Codecs
    ///

    /// VP8 Video Codec
    pub const VIDEO_VP8: Fourcc = Fourcc(*b"vp08");

    /// VP9 Video Codec
    pub const VIDEO_VP9: Fourcc = Fourcc(*b"vp09");

    /// AV1 Video Codec
    pub const VIDEO_AV1: Fourcc = Fourcc(*b"av01");

    /// AVC (H264) Video Codec
    pub const VIDEO_AVC: Fourcc = Fourcc(*b"avc1");

    /// HEVC (H265) Video Codec
    pub const VIDEO_HEVC: Fourcc = Fourcc(*b"hvc1");

    /// MJPEG Codec
    pub const VIDEO_MJPEG: Fourcc = Fourcc(*b"MJPG");

    /// JPEG Codec
    pub const VIDEO_JPEG: Fourcc = Fourcc(*b"JPEG");

    /// Pixel formats

    /// | C1 C2 C3 C4 C5 C6 C7 C8 |
    pub const PIXEL_FORMAT_C1: Fourcc = Fourcc(*b"C1  ");

    /// | C1 C1 C2 C2 C3 C3 C4 C4 |
    pub const PIXEL_FORMAT_C2: Fourcc = Fourcc(*b"C2  ");

    /// | C1 C1 C1 C1 C2 C2 C2 C2 |
    pub const PIXEL_FORMAT_C4: Fourcc = Fourcc(*b"C4  ");

    /// | C C C C C C C C |
    pub const PIXEL_FORMAT_C8: Fourcc = Fourcc(*b"C8  ");

    ///
    /// Darkness (inverse relationship between channel value and brightness)
    ///
    /// | D1 D2 D3 D4 D5 D6 D7 D8 |
    pub const PIXEL_FORMAT_D1: Fourcc = Fourcc(*b"D1  ");

    /// | D1 D1 D2 D2 D3 D3 D4 D4 |
    pub const PIXEL_FORMAT_D2: Fourcc = Fourcc(*b"D2  ");

    /// | C1 C1 C1 C1 C2 C2 C2 C2 |
    pub const PIXEL_FORMAT_D4: Fourcc = Fourcc(*b"D4  ");

    /// | D D D D D D D D |
    pub const PIXEL_FORMAT_D8: Fourcc = Fourcc(*b"D8  ");

    ///
    /// Red (direct relationship between channel value and brightness)
    ///
    /// | R1 R2 R3 R4 R5 R6 R7 R8 |
    pub const PIXEL_FORMAT_R1: Fourcc = Fourcc(*b"R1  ");

    /// | R1 R1 R2 R2 R3 R3 R4 R4 |
    pub const PIXEL_FORMAT_R2: Fourcc = Fourcc(*b"R2  ");

    /// | R1 R1 R1 R1 R2 R2 R2 R2 |
    pub const PIXEL_FORMAT_R4: Fourcc = Fourcc(*b"R4  ");

    /// | R R R R R R R R |
    pub const PIXEL_FORMAT_R8: Fourcc = Fourcc(*b"R8  ");

    /// 10 bpp Red (direct relationship between channel value and brightness)
    pub const PIXEL_FORMAT_R10: Fourcc = Fourcc(*b"R10 "); // [15:0] x:R 6:10 little endian 

    /// 12 bpp Red (direct relationship between channel value and brightness)
    pub const PIXEL_FORMAT_R12: Fourcc = Fourcc(*b"R12 "); // [15:0] x:R 4:12 little endian 

    /// 16 bpp Red (direct relationship between channel value and brightness)
    pub const PIXEL_FORMAT_R16: Fourcc = Fourcc(*b"R16 "); // [15:0] R little endian 

    /// 16 bpp RG
    pub const PIXEL_FORMAT_RG88: Fourcc = Fourcc(*b"RG88"); // [15:0] R:G 8:8 little endian 
    pub const PIXEL_FORMAT_GR88: Fourcc = Fourcc(*b"GR88"); // [15:0] G:R 8:8 little endian 

    /// 32 bpp RG
    pub const PIXEL_FORMAT_RG1616: Fourcc = Fourcc(*b"RG32"); // [31:0] R:G 16:16 little endian 
    pub const PIXEL_FORMAT_GR1616: Fourcc = Fourcc(*b"GR32"); // [31:0] G:R 16:16 little endian 

    ///
    /// 8 bpp RGB
    ///
    /// | R R R G G G B B |
    pub const PIXEL_FORMAT_RGB332: Fourcc = Fourcc(*b"RGB8");

    /// | B B G G G R R R |
    pub const PIXEL_FORMAT_BGR233: Fourcc = Fourcc(*b"BGR8");

    /// 16 bpp RGB
    pub const PIXEL_FORMAT_XRGB4444: Fourcc = Fourcc(*b"XR12"); // [15:0] x:R:G:B 4:4:4:4 little endian 
    pub const PIXEL_FORMAT_XBGR4444: Fourcc = Fourcc(*b"XB12"); // [15:0] x:B:G:R 4:4:4:4 little endian 
    pub const PIXEL_FORMAT_RGBX4444: Fourcc = Fourcc(*b"RX12"); // [15:0] R:G:B:x 4:4:4:4 little endian 
    pub const PIXEL_FORMAT_BGRX4444: Fourcc = Fourcc(*b"BX12"); // [15:0] B:G:R:x 4:4:4:4 little endian 

    pub const PIXEL_FORMAT_ARGB4444: Fourcc = Fourcc(*b"AR12"); // [15:0] A:R:G:B 4:4:4:4 little endian 
    pub const PIXEL_FORMAT_ABGR4444: Fourcc = Fourcc(*b"AB12"); // [15:0] A:B:G:R 4:4:4:4 little endian 
    pub const PIXEL_FORMAT_RGBA4444: Fourcc = Fourcc(*b"RA12"); // [15:0] R:G:B:A 4:4:4:4 little endian 
    pub const PIXEL_FORMAT_BGRA4444: Fourcc = Fourcc(*b"BA12"); // [15:0] B:G:R:A 4:4:4:4 little endian 

    pub const PIXEL_FORMAT_XRGB1555: Fourcc = Fourcc(*b"XR15"); // [15:0] x:R:G:B 1:5:5:5 little endian 
    pub const PIXEL_FORMAT_XBGR1555: Fourcc = Fourcc(*b"XB15"); // [15:0] x:B:G:R 1:5:5:5 little endian 
    pub const PIXEL_FORMAT_RGBX5551: Fourcc = Fourcc(*b"RX15"); // [15:0] R:G:B:x 5:5:5:1 little endian 
    pub const PIXEL_FORMAT_BGRX5551: Fourcc = Fourcc(*b"BX15"); // [15:0] B:G:R:x 5:5:5:1 little endian 

    pub const PIXEL_FORMAT_ARGB1555: Fourcc = Fourcc(*b"AR15"); // [15:0] A:R:G:B 1:5:5:5 little endian 
    pub const PIXEL_FORMAT_ABGR1555: Fourcc = Fourcc(*b"AB15"); // [15:0] A:B:G:R 1:5:5:5 little endian 
    pub const PIXEL_FORMAT_RGBA5551: Fourcc = Fourcc(*b"RA15"); // [15:0] R:G:B:A 5:5:5:1 little endian 
    pub const PIXEL_FORMAT_BGRA5551: Fourcc = Fourcc(*b"BA15"); // [15:0] B:G:R:A 5:5:5:1 little endian 

    pub const PIXEL_FORMAT_RGB565: Fourcc = Fourcc(*b"RG16"); // [15:0] R:G:B 5:6:5 little endian 
    pub const PIXEL_FORMAT_BGR565: Fourcc = Fourcc(*b"BG16"); // [15:0] B:G:R 5:6:5 little endian 

    // 24 bpp RGB
    pub const PIXEL_FORMAT_RGB888: Fourcc = Fourcc(*b"RG24"); // [23:0] R:G:B little endian 
    pub const PIXEL_FORMAT_BGR888: Fourcc = Fourcc(*b"BG24"); // [23:0] B:G:R little endian 

    // 32 bpp RGB
    pub const PIXEL_FORMAT_XRGB8888: Fourcc = Fourcc(*b"XR24"); // [31:0] x:R:G:B 8:8:8:8 little endian 
    pub const PIXEL_FORMAT_XBGR8888: Fourcc = Fourcc(*b"XB24"); // [31:0] x:B:G:R 8:8:8:8 little endian 
    pub const PIXEL_FORMAT_RGBX8888: Fourcc = Fourcc(*b"RX24"); // [31:0] R:G:B:x 8:8:8:8 little endian 
    pub const PIXEL_FORMAT_BGRX8888: Fourcc = Fourcc(*b"BX24"); // [31:0] B:G:R:x 8:8:8:8 little endian 

    pub const PIXEL_FORMAT_ARGB8888: Fourcc = Fourcc(*b"AR24"); // [31:0] A:R:G:B 8:8:8:8 little endian 
    pub const PIXEL_FORMAT_ABGR8888: Fourcc = Fourcc(*b"AB24"); // [31:0] A:B:G:R 8:8:8:8 little endian 
    pub const PIXEL_FORMAT_RGBA8888: Fourcc = Fourcc(*b"RA24"); // [31:0] R:G:B:A 8:8:8:8 little endian 
    pub const PIXEL_FORMAT_BGRA8888: Fourcc = Fourcc(*b"BA24"); // [31:0] B:G:R:A 8:8:8:8 little endian 

    pub const PIXEL_FORMAT_XRGB2101010: Fourcc = Fourcc(*b"XR30"); // [31:0] x:R:G:B 2:10:10:10 little endian 
    pub const PIXEL_FORMAT_XBGR2101010: Fourcc = Fourcc(*b"XB30"); // [31:0] x:B:G:R 2:10:10:10 little endian 
    pub const PIXEL_FORMAT_RGBX1010102: Fourcc = Fourcc(*b"RX30"); // [31:0] R:G:B:x 10:10:10:2 little endian 
    pub const PIXEL_FORMAT_BGRX1010102: Fourcc = Fourcc(*b"BX30"); // [31:0] B:G:R:x 10:10:10:2 little endian 

    pub const PIXEL_FORMAT_ARGB2101010: Fourcc = Fourcc(*b"AR30"); // [31:0] A:R:G:B 2:10:10:10 little endian 
    pub const PIXEL_FORMAT_ABGR2101010: Fourcc = Fourcc(*b"AB30"); // [31:0] A:B:G:R 2:10:10:10 little endian 
    pub const PIXEL_FORMAT_RGBA1010102: Fourcc = Fourcc(*b"RA30"); // [31:0] R:G:B:A 10:10:10:2 little endian 
    pub const PIXEL_FORMAT_BGRA1010102: Fourcc = Fourcc(*b"BA30"); // [31:0] B:G:R:A 10:10:10:2 little endian 

    // 64 bpp RGB
    pub const PIXEL_FORMAT_XRGB16161616: Fourcc = Fourcc(*b"XR48"); // [63:0] x:R:G:B 16:16:16:16 little endian 
    pub const PIXEL_FORMAT_XBGR16161616: Fourcc = Fourcc(*b"XB48"); // [63:0] x:B:G:R 16:16:16:16 little endian 

    pub const PIXEL_FORMAT_ARGB16161616: Fourcc = Fourcc(*b"AR48"); // [63:0] A:R:G:B 16:16:16:16 little endian 
    pub const PIXEL_FORMAT_ABGR16161616: Fourcc = Fourcc(*b"AB48"); // [63:0] A:B:G:R 16:16:16:16 little endian 

    ///
    /// Floating point 64bpp RGB
    /// IEEE 754-2008 binary16 half-precision float
    /// [15:0] sign:exponent:mantissa 1:5:10
    ///
    pub const PIXEL_FORMAT_XRGB16161616F: Fourcc = Fourcc(*b"XR4H"); // [63:0] x:R:G:B 16:16:16:16 little endian 
    pub const PIXEL_FORMAT_XBGR16161616F: Fourcc = Fourcc(*b"XB4H"); // [63:0] x:B:G:R 16:16:16:16 little endian 

    pub const PIXEL_FORMAT_ARGB16161616F: Fourcc = Fourcc(*b"AR4H"); // [63:0] A:R:G:B 16:16:16:16 little endian 
    pub const PIXEL_FORMAT_ABGR16161616F: Fourcc = Fourcc(*b"AB4H"); // [63:0] A:B:G:R 16:16:16:16 little endian 

    ///
    /// RGBA format with 10-bit components packed in 64-bit per pixel, with 6 bits
    /// of unused padding per component:
    ///
    pub const PIXEL_FORMAT_AXBXGXRX106106106106: Fourcc = Fourcc(*b"AB10"); // [63:0] A:x:B:x:G:x:R:x 10:6:10:6:10:6:10:6 little endian 

    // packed YCbCr
    pub const PIXEL_FORMAT_YUYV: Fourcc = Fourcc(*b"YUYV"); // [31:0] Cr0:Y1:Cb0:Y0 8:8:8:8 little endian 
    pub const PIXEL_FORMAT_YVYU: Fourcc = Fourcc(*b"YVYU"); // [31:0] Cb0:Y1:Cr0:Y0 8:8:8:8 little endian 
    pub const PIXEL_FORMAT_UYVY: Fourcc = Fourcc(*b"UYVY"); // [31:0] Y1:Cr0:Y0:Cb0 8:8:8:8 little endian 
    pub const PIXEL_FORMAT_VYUY: Fourcc = Fourcc(*b"VYUY"); // [31:0] Y1:Cb0:Y0:Cr0 8:8:8:8 little endian 

    pub const PIXEL_FORMAT_AYUV: Fourcc = Fourcc(*b"AYUV"); // [31:0] A:Y:Cb:Cr 8:8:8:8 little endian 
    pub const PIXEL_FORMAT_AVUY8888: Fourcc = Fourcc(*b"AVUY"); // [31:0] A:Cr:Cb:Y 8:8:8:8 little endian 
    pub const PIXEL_FORMAT_XYUV8888: Fourcc = Fourcc(*b"XYUV"); // [31:0] X:Y:Cb:Cr 8:8:8:8 little endian 
    pub const PIXEL_FORMAT_XVUY8888: Fourcc = Fourcc(*b"XVUY"); // [31:0] X:Cr:Cb:Y 8:8:8:8 little endian 
    pub const PIXEL_FORMAT_VUY888: Fourcc = Fourcc(*b"VU24"); // [23:0] Cr:Cb:Y 8:8:8 little endian 
    pub const PIXEL_FORMAT_VUY101010: Fourcc = Fourcc(*b"VU30"); // Y followed by U then V, 10:10:10. Non-linear modifier only 

    /*
     * packed Y2xx indicate for each component, xx valid data occupy msb
     * 16-xx padding occupy lsb
     */
    pub const PIXEL_FORMAT_Y210: Fourcc = Fourcc(*b"Y210"); // [63:0] Cr0:0:Y1:0:Cb0:0:Y0:0 10:6:10:6:10:6:10:6 little endian per 2 Y pixels 
    pub const PIXEL_FORMAT_Y212: Fourcc = Fourcc(*b"Y212"); // [63:0] Cr0:0:Y1:0:Cb0:0:Y0:0 12:4:12:4:12:4:12:4 little endian per 2 Y pixels 
    pub const PIXEL_FORMAT_Y216: Fourcc = Fourcc(*b"Y216"); // [63:0] Cr0:Y1:Cb0:Y0 16:16:16:16 little endian per 2 Y pixels 

    /*
     * packed Y4xx indicate for each component, xx valid data occupy msb
     * 16-xx padding occupy lsb except Y410
     */
    pub const PIXEL_FORMAT_Y410: Fourcc = Fourcc(*b"Y410"); // [31:0] A:Cr:Y:Cb 2:10:10:10 little endian 
    pub const PIXEL_FORMAT_Y412: Fourcc = Fourcc(*b"Y412"); // [63:0] A:0:Cr:0:Y:0:Cb:0 12:4:12:4:12:4:12:4 little endian 
    pub const PIXEL_FORMAT_Y416: Fourcc = Fourcc(*b"Y416"); // [63:0] A:Cr:Y:Cb 16:16:16:16 little endian 

    pub const PIXEL_FORMAT_XVYU2101010: Fourcc = Fourcc(*b"XV30"); // [31:0] X:Cr:Y:Cb 2:10:10:10 little endian 
    pub const PIXEL_FORMAT_XVYU12_16161616: Fourcc = Fourcc(*b"XV36"); // [63:0] X:0:Cr:0:Y:0:Cb:0 12:4:12:4:12:4:12:4 little endian 
    pub const PIXEL_FORMAT_XVYU16161616: Fourcc = Fourcc(*b"XV48"); // [63:0] X:Cr:Y:Cb 16:16:16:16 little endian 

    ///
    /// 1-plane YUV 4:2:0
    /// In these formats, the component ordering is specified (Y, followed by U
    /// then V), but the exact Linear layout is undefined.
    /// These formats can only be used with a non-Linear modifier.
    ///
    pub const PIXEL_FORMAT_YUV420_8BIT: Fourcc = Fourcc(*b"YU08");
    pub const PIXEL_FORMAT_YUV420_10BIT: Fourcc = Fourcc(*b"YU10");

    ///
    /// 2 plane RGB + A
    /// index 0 = RGB plane, same format as the corresponding non _A8 format has
    /// index 1 = A plane, [7:0] A
    ///
    pub const PIXEL_FORMAT_XRGB8888_A8: Fourcc = Fourcc(*b"XRA8");
    pub const PIXEL_FORMAT_XBGR8888_A8: Fourcc = Fourcc(*b"XBA8");
    pub const PIXEL_FORMAT_RGBX8888_A8: Fourcc = Fourcc(*b"RXA8");
    pub const PIXEL_FORMAT_BGRX8888_A8: Fourcc = Fourcc(*b"BXA8");
    pub const PIXEL_FORMAT_RGB888_A8: Fourcc = Fourcc(*b"R8A8");
    pub const PIXEL_FORMAT_BGR888_A8: Fourcc = Fourcc(*b"B8A8");
    pub const PIXEL_FORMAT_RGB565_A8: Fourcc = Fourcc(*b"R5A8");
    pub const PIXEL_FORMAT_BGR565_A8: Fourcc = Fourcc(*b"B5A8");

    ///
    /// 2 plane YCbCr
    /// index 0 = Y plane, [7:0] Y
    /// index 1 = Cr:Cb plane, [15:0] Cr:Cb little endian
    /// or
    /// index 1 = Cb:Cr plane, [15:0] Cb:Cr little endian
    ///
    pub const PIXEL_FORMAT_NV12: Fourcc = Fourcc(*b"NV12"); // 2x2 subsampled Cr:Cb plane 
    pub const PIXEL_FORMAT_NV21: Fourcc = Fourcc(*b"NV21"); // 2x2 subsampled Cb:Cr plane 
    pub const PIXEL_FORMAT_NV16: Fourcc = Fourcc(*b"NV16"); // 2x1 subsampled Cr:Cb plane 
    pub const PIXEL_FORMAT_NV61: Fourcc = Fourcc(*b"NV61"); // 2x1 subsampled Cb:Cr plane 
    pub const PIXEL_FORMAT_NV24: Fourcc = Fourcc(*b"NV24"); // non-subsampled Cr:Cb plane 
    pub const PIXEL_FORMAT_NV42: Fourcc = Fourcc(*b"NV42"); // non-subsampled Cb:Cr plane 
    ///
    /// 2 plane YCbCr
    /// index 0 = Y plane, [39:0] Y3:Y2:Y1:Y0 little endian
    /// index 1 = Cr:Cb plane, [39:0] Cr1:Cb1:Cr0:Cb0 little endian
    ///
    pub const PIXEL_FORMAT_NV15: Fourcc = Fourcc(*b"NV15"); // 2x2 subsampled Cr:Cb plane 
    pub const PIXEL_FORMAT_NV20: Fourcc = Fourcc(*b"NV20"); // 2x1 subsampled Cr:Cb plane 
    pub const PIXEL_FORMAT_NV30: Fourcc = Fourcc(*b"NV30"); // non-subsampled Cr:Cb plane 

    ///
    /// 2 plane YCbCr MSB aligned
    /// index 0 = Y plane, [15:0] Y:x [10:6] little endian
    /// index 1 = Cr:Cb plane, [31:0] Cr:x:Cb:x [10:6:10:6] little endian
    ///
    pub const PIXEL_FORMAT_P210: Fourcc = Fourcc(*b"P210"); // 2x1 subsampled Cr:Cb plane, 10 bit per channel 

    ///
    /// 2 plane YCbCr MSB aligned
    /// index 0 = Y plane, [15:0] Y:x [10:6] little endian
    /// index 1 = Cr:Cb plane, [31:0] Cr:x:Cb:x [10:6:10:6] little endian
    ///
    pub const PIXEL_FORMAT_P010: Fourcc = Fourcc(*b"P010"); // 2x2 subsampled Cr:Cb plane 10 bits per channel 

    ///
    /// 2 plane YCbCr MSB aligned
    /// index 0 = Y plane, [15:0] Y:x [12:4] little endian
    /// index 1 = Cr:Cb plane, [31:0] Cr:x:Cb:x [12:4:12:4] little endian
    ///
    pub const PIXEL_FORMAT_P012: Fourcc = Fourcc(*b"P012"); // 2x2 subsampled Cr:Cb plane 12 bits per channel 

    ///
    /// 2 plane YCbCr MSB aligned
    /// index 0 = Y plane, [15:0] Y little endian
    /// index 1 = Cr:Cb plane, [31:0] Cr:Cb [16:16] little endian
    ///
    pub const PIXEL_FORMAT_P016: Fourcc = Fourcc(*b"P016"); // 2x2 subsampled Cr:Cb plane 16 bits per channel 

    ///
    /// 2 plane YCbCr42
    /// 3 10 bit components and 2 padding bits packed into 4 bytes.
    /// index 0 = Y plane, [31:0] x:Y2:Y1:Y0 2:10:10:10 little endian
    /// index 1 = Cr:Cb plane, [63:0] x:Cr2:Cb2:Cr1:x:Cb1:Cr0:Cb0 [2:10:10:10:2:10:10:10] little endian
    ///
    pub const PIXEL_FORMAT_P030: Fourcc = Fourcc(*b"P030"); // 2x2 subsampled Cr:Cb plane 10 bits per channel packed 

    ///
    /// 3 plane non-subsampled (444) YCb
    /// 16 bits per component, but only 10 bits are used and 6 bits are padded
    /// index 0: Y plane, [15:0] Y:x [10:6] little endian
    /// index 1: Cb plane, [15:0] Cb:x [10:6] little endian
    /// index 2: Cr plane, [15:0] Cr:x [10:6] little endian
    ///
    pub const PIXEL_FORMAT_Q410: Fourcc = Fourcc(*b"Q410");

    ///
    /// 3 plane non-subsampled (444) YCr
    /// 16 bits per component, but only 10 bits are used and 6 bits are padded
    /// index 0: Y plane, [15:0] Y:x [10:6] little endian
    /// index 1: Cr plane, [15:0] Cr:x [10:6] little endian
    /// index 2: Cb plane, [15:0] Cb:x [10:6] little endian
    ///
    pub const PIXEL_FORMAT_Q401: Fourcc = Fourcc(*b"Q401");

    ///
    /// 3 plane YCbCr
    /// index 0: Y plane, [7:0] Y
    /// index 1: Cb plane, [7:0] Cb
    /// index 2: Cr plane, [7:0] Cr
    /// or
    /// index 1: Cr plane, [7:0] Cr
    /// index 2: Cb plane, [7:0] Cb
    ///
    pub const PIXEL_FORMAT_YUV410: Fourcc = Fourcc::from_static("YUV9"); // 4x4 subsampled Cb (1) and Cr (2) planes 
    pub const PIXEL_FORMAT_YVU410: Fourcc = Fourcc::from_static("YVU9"); // 4x4 subsampled Cr (1) and Cb (2) planes 
    pub const PIXEL_FORMAT_YUV411: Fourcc = Fourcc::from_static("YU11"); // 4x1 subsampled Cb (1) and Cr (2) planes 
    pub const PIXEL_FORMAT_YVU411: Fourcc = Fourcc::from_static("YV11"); // 4x1 subsampled Cr (1) and Cb (2) planes 
    pub const PIXEL_FORMAT_YUV420: Fourcc = Fourcc::from_static("YU12"); // 2x2 subsampled Cb (1) and Cr (2) planes 
    pub const PIXEL_FORMAT_YVU420: Fourcc = Fourcc::from_static("YV12"); // 2x2 subsampled Cr (1) and Cb (2) planes 
    pub const PIXEL_FORMAT_YUV422: Fourcc = Fourcc::from_static("YU16"); // 2x1 subsampled Cb (1) and Cr (2) planes 
    pub const PIXEL_FORMAT_YVU422: Fourcc = Fourcc::from_static("YV16"); // 2x1 subsampled Cr (1) and Cb (2) planes 
    pub const PIXEL_FORMAT_YUV444: Fourcc = Fourcc::from_static("YU24"); // non-subsampled Cb (1) and Cr (2) planes 
    pub const PIXEL_FORMAT_YVU444: Fourcc = Fourcc::from_static("YV24"); // non-subsampled Cr (1) and Cb (2) planes 

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

impl From<[u8; 4]> for Fourcc {
    fn from(n: [u8; 4]) -> Self {
        Self(n)
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

impl std::fmt::Debug for Fourcc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c: [u8; 4] = (*self).into();

        f.write_fmt(format_args!(
            "Fourcc(['{}', '{}', '{}', '{}'])",
            c[0] as char, c[1] as char, c[2] as char, c[3] as char
        ))
    }
}
