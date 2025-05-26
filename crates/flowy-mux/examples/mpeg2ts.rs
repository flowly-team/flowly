use std::{path::PathBuf, pin::pin, str::FromStr};

use bytes::Bytes;
use flowy_core::Frame;
use flowy_mux::mpeg2ts::Mpeg2TsDemux;
use flowy_service::{Service, ServiceExt, pipeline};
use futures::TryStreamExt;
use tokio::io::AsyncReadExt;

pub struct FileReader;

impl<E: std::error::Error + Send + Sync + 'static> Service<Result<PathBuf, E>> for FileReader {
    type Out = std::io::Result<Bytes>;

    fn handle(
        self,
        input: impl futures::Stream<Item = Result<PathBuf, E>> + Send,
    ) -> impl futures::Stream<Item = Self::Out> + Send {
        async_stream::try_stream! {
            let mut input = pin!(input);
            let mut buf = vec![0u8; 188 * 1024];

            while let Some(path) = input.try_next().await.map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, Box::new(err)))?  {
                let mut file = tokio::fs::File::open(path).await?;
                loop {
                    yield match file.read(&mut buf[..]).await? {
                        0 => break,
                        n => buf[0..n].to_vec().into()
                    };
                }
            }
        }
    }
}

/// NAL unit type, as in T.REC H.265 Table 7-1.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(u8)]
pub enum UnitType {
    TrailN = 0,
    TrailR = 1,
    TsaN = 2,
    TsaR = 3,
    StsaN = 4,
    StsaR = 5,
    RadlN = 6,
    RadlR = 7,
    RaslN = 8,
    RaslR = 9,
    RsvVclN10 = 10,
    RsvVclR11 = 11,
    RsvVclN12 = 12,
    RsvVclR13 = 13,
    RsvVclN14 = 14,
    RsvVclR15 = 15,
    BlaWLp = 16,
    BlaWRadl = 17,
    BlaNLp = 18,
    IdrWRadl = 19,
    IdrNLp = 20,
    CraNut = 21,
    RsvIrapVcl22 = 22,
    RsvIrapVcl23 = 23,
    RsvVcl24 = 24,
    RsvVcl25 = 25,
    RsvVcl26 = 26,
    RsvVcl27 = 27,
    RsvVcl28 = 28,
    RsvVcl29 = 29,
    RsvVcl30 = 30,
    RsvVcl31 = 31,
    VpsNut = 32,
    SpsNut = 33,
    PpsNut = 34,

    /// Access unit delimiter.
    AudNut = 35,

    /// End of sequence.
    EosNut = 36,

    /// End of bitstream.
    EobNut = 37,
    FdNut = 38,
    PrefixSeiNut = 39,
    SuffixSeiNut = 40,
    RsvNvcl41 = 41,
    RsvNvcl42 = 42,
    RsvNvcl43 = 43,
    RsvNvcl44 = 44,
    RsvNvcl45 = 45,
    RsvNvcl46 = 46,
    RsvNvcl47 = 47,
    Unspec48 = 48,
    Unspec49 = 49,
    Unspec50 = 50,
    Unspec51 = 51,
    Unspec52 = 52,
    Unspec53 = 53,
    Unspec54 = 54,
    Unspec55 = 55,
    Unspec56 = 56,
    Unspec57 = 57,
    Unspec58 = 58,
    Unspec59 = 59,
    Unspec60 = 60,
    Unspec61 = 61,
    Unspec62 = 62,
    Unspec63 = 63,
}

impl TryFrom<u8> for UnitType {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > 63 {
            anyhow::bail!("NAL 0x{:02X} is out of range", value)
        }

        // SAFETY: `UnitType` is `repr(u8)` and C-like; `value` is in range.
        Ok(unsafe { std::mem::transmute::<u8, UnitType>(value) })
    }
}

impl From<UnitType> for u8 {
    fn from(t: UnitType) -> u8 {
        // SAFETY: `UnitType` is `repr(u8)` and C-like.
        unsafe { std::mem::transmute(t) }
    }
}

/// `nal_unit_header` as in T.REC H.265 section 7.3.1.2.
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Header([u8; 2]);

impl std::fmt::Debug for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Header")
            .field("unit_type", &self.unit_type())
            .field("nuh_layer_id", &self.nuh_layer_id())
            .field("nuh_temporal_id_plus1", &self.nuh_temporal_id_plus1())
            .finish()
    }
}

impl TryFrom<[u8; 2]> for Header {
    type Error = anyhow::Error;

    fn try_from(value: [u8; 2]) -> Result<Self, Self::Error> {
        if (value[0] & 0b1000_0000) != 0 {
            anyhow::bail!(
                "forbidden zero bit is set in NAL header 0x{:02X}{:02X}",
                value[0],
                value[1]
            );
        }
        if (value[1] & 0b111) == 0 {
            anyhow::bail!(
                "zero temporal_id_plus1 in NAL header 0x{:02X}{:02X}",
                value[0],
                value[1]
            );
        }
        Ok(Self(value))
    }
}

impl std::ops::Deref for Header {
    type Target = [u8; 2];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Header {
    /// Returns a new header of the given unit type.
    pub fn with_unit_type(self, t: UnitType) -> Self {
        let mut out = self.0;
        out[0] = (out[0] & 0b1000_0001) | (u8::from(t) << 1);
        Self(out)
    }

    /// The NAL unit type.
    pub fn unit_type(self) -> UnitType {
        UnitType::try_from(self.0[0] >> 1).expect("6-bit value must be valid NAL type")
    }

    /// The `nuh_layer_id`, as a 6-bit value.
    pub fn nuh_layer_id(self) -> u8 {
        (self.0[0] & 0b1) << 5 | (self.0[1] >> 3)
    }

    /// The `num_temporal_id_plus1`, as a non-zero 3-bit value.
    pub fn nuh_temporal_id_plus1(self) -> u8 {
        self.0[1] & 0b111
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let reader = pipeline().pipe(FileReader).pipe(Mpeg2TsDemux::new());

    let stream = reader.handle(futures::stream::once(async move {
        PathBuf::from_str("/home/andrey/demo/h265/street.ts")
    }));

    let mut stream = pin!(stream);

    while let Some(block) = stream.try_next().await? {
        println!();

        for unit in block.units() {
            let header = Header::try_from([unit[0], unit[1]]).unwrap();

            println!(
                "{}\t{:0.2}\t{}\t {:?} {}",
                block.seq(),
                (block.pts() as f64) / 1_000_000.0,
                block.is_key_frame(),
                header.unit_type(),
                unit.len()
            );
        }
    }

    Ok(())
}
