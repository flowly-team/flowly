use arrayvec::ArrayVec;
pub use bytes::TryGetError;
use bytes::{Buf, BufMut, Bytes, BytesMut};

pub trait Reader: Buf {
    fn read_string(&mut self, len: usize) -> Result<String, TryGetError> {
        let mut buf = BytesMut::with_capacity(len);
        buf.put(Buf::take(self, len));

        String::from_utf8(buf.into()).map_err(|_| TryGetError {
            requested: 0,
            available: 0,
        })
    }

    fn read_array<const N: usize>(&mut self, len: usize) -> Result<ArrayVec<u8, N>, TryGetError> {
        let mut arr = ArrayVec::new();
        unsafe { arr.set_len(usize::min(N, len)) };
        self.read_slice(&mut arr)?;

        Ok(arr)
    }

    fn read_slice(&mut self, dst: &mut [u8]) -> Result<(), TryGetError> {
        self.try_copy_to_slice(dst)
    }

    fn read_bytes(&mut self, len: usize) -> Result<Bytes, TryGetError> {
        let available = Self::remaining(self);
        if available < len {
            Err(TryGetError {
                requested: len,
                available,
            })
        } else {
            Ok(self.copy_to_bytes(len))
        }
    }

    fn read_bytes_prepend(&mut self, len: usize, prep: &[u8]) -> Result<Bytes, TryGetError> {
        let available = Self::remaining(self);

        if available < len {
            Err(TryGetError {
                requested: len,
                available,
            })
        } else {
            let mut ret = BytesMut::with_capacity(len + prep.len());
            ret.put(prep);
            ret.put(self.take(len));

            Ok(ret.freeze())
        }
    }

    #[inline]
    fn read_u8(&mut self) -> Result<u8, TryGetError> {
        self.try_get_u8()
    }

    #[inline]
    fn read_i8(&mut self) -> Result<i8, TryGetError> {
        self.try_get_i8()
    }

    #[inline]
    fn read_u16(&mut self) -> Result<u16, TryGetError> {
        self.try_get_u16()
    }

    #[inline]
    fn read_i16(&mut self) -> Result<i16, TryGetError> {
        self.try_get_i16()
    }

    #[inline]
    fn read_u32(&mut self) -> Result<u32, TryGetError> {
        self.try_get_u32()
    }

    #[inline]
    fn read_i32(&mut self) -> Result<i32, TryGetError> {
        self.try_get_i32()
    }

    #[inline]
    fn read_u64(&mut self) -> Result<u64, TryGetError> {
        self.try_get_u64()
    }

    #[inline]
    fn read_i64(&mut self) -> Result<i64, TryGetError> {
        self.try_get_i64()
    }

    #[inline]
    fn read_uint(&mut self, nbytes: usize) -> Result<u64, TryGetError> {
        self.try_get_uint(nbytes)
    }

    #[inline]
    fn read_int(&mut self, nbytes: usize) -> Result<i64, TryGetError> {
        self.try_get_int(nbytes)
    }

    #[inline]
    fn read_f32(&mut self) -> Result<f32, TryGetError> {
        self.try_get_f32()
    }

    #[inline]
    fn read_f64(&mut self) -> Result<f64, TryGetError> {
        self.try_get_f64()
    }

    #[inline]
    fn peek_u8(&mut self) -> Result<u8, TryGetError> {
        self.chunk().read_u8()
    }

    #[inline]
    fn read_i24(&mut self) -> Result<i32, TryGetError> {
        Ok(self.read_int(3)? as i32)
    }

    #[inline]
    fn read_u24(&mut self) -> Result<u32, TryGetError> {
        Ok(self.read_uint(3)? as u32)
    }

    #[inline]
    fn read_i48(&mut self) -> Result<i64, TryGetError> {
        self.read_int(6)
    }

    #[inline]
    fn read_u48(&mut self) -> Result<u64, TryGetError> {
        self.read_uint(6)
    }

    #[inline]
    fn read_remaining(&mut self) -> Bytes {
        self.read_bytes(self.remaining()).unwrap()
    }
}

impl<T: Buf> Reader for T {}

pub trait ReaderExt: Sized + Reader {
    #[inline]
    fn read_u8p2<const A: u8, const B: u8>(&mut self) -> Result<(u8, u8), TryGetError> {
        const {
            assert! {A as usize + B as usize == 8};
        }

        let val = self.read_u8()?;
        let mask = (((1u16 << A) - 1) << B) as u8;

        Ok((val >> B, val & (!mask)))
    }

    #[inline]
    fn read_u8p3<const A: u8, const B: u8, const C: u8>(
        &mut self,
    ) -> Result<(u8, u8, u8), TryGetError> {
        const {
            assert! {A as usize + B as usize + C as usize == 8};
        }

        let val = self.read_u8()?;
        let b_mask = ((1u16 << B) - 1) as u8;
        let c_mask = ((1u16 << C) - 1) as u8;

        Ok(((val >> (B + C)), (val >> C) & b_mask, val & c_mask))
    }

    #[inline]
    fn read_u8p4<const A: u8, const B: u8, const C: u8, const D: u8>(
        &mut self,
    ) -> Result<(u8, u8, u8, u8), TryGetError> {
        const {
            assert! {A as usize + B as usize + C as usize + D as usize == 8};
        }

        let val = self.read_u8()?;
        let b_mask = ((1u16 << B) - 1) as u8;
        let c_mask = ((1u16 << C) - 1) as u8;
        let d_mask = ((1u16 << D) - 1) as u8;

        Ok((
            (val >> (B + C + D)),
            (val >> (C + D)) & b_mask,
            (val >> D) & c_mask,
            val & d_mask,
        ))
    }

    #[inline]
    fn read_u16p2<const A: u8, const B: u8>(&mut self) -> Result<(u16, u16), TryGetError> {
        const {
            assert! {A as usize + B as usize == 16};
        }

        let val = self.read_u16()?;
        let mask = (((1u32 << A) - 1) << B) as u16;

        Ok((val >> B, val & (!mask)))
    }

    #[inline]
    fn read_u16p3<const A: u8, const B: u8, const C: u8>(
        &mut self,
    ) -> Result<(u16, u16, u16), TryGetError> {
        const {
            assert! {A as usize + B as usize + C as usize == 16};
        }

        let val = self.read_u16()?;
        let b_mask = ((1u32 << B) - 1) as u16;
        let c_mask = ((1u32 << C) - 1) as u16;

        Ok(((val >> (B + C)), (val >> C) & b_mask, val & c_mask))
    }

    #[inline]
    fn read_u16p4<const A: u16, const B: u16, const C: u16, const D: u16>(
        &mut self,
    ) -> Result<(u16, u16, u16, u16), TryGetError> {
        const {
            assert! {A as usize + B as usize + C as usize + D as usize == 16};
        }

        let val = self.read_u16()?;
        let b_mask = ((1u32 << B) - 1) as u16;
        let c_mask = ((1u32 << C) - 1) as u16;
        let d_mask = ((1u32 << D) - 1) as u16;

        Ok((
            (val >> (B + C + D)),
            (val >> (C + D)) & b_mask,
            (val >> D) & c_mask,
            val & d_mask,
        ))
    }
}

impl<T: Reader> ReaderExt for T {}

pub trait Writer: BufMut {
    #[inline]
    fn put_i24(&mut self, val: i32) {
        self.put_int(val as i64, 3)
    }

    #[inline]
    fn put_u24(&mut self, val: u32) {
        self.put_uint(val as u64, 3)
    }

    #[inline]
    fn put_i48(&mut self, val: i64) {
        self.put_int(val, 6)
    }

    #[inline]
    fn put_u48(&mut self, val: u64) {
        self.put_uint(val, 6)
    }

    #[inline]
    fn put_str(&mut self, string: &str) {
        self.put_slice(string.as_bytes())
    }
}

impl<T: BufMut> Writer for T {}

pub trait WriterExt: Writer {
    fn put_u8p2<const A: u8, const B: u8>(&mut self, a: u8, b: u8) {
        let _ = b;
        let _ = a;
        const {
            assert!(A + B == 8);
        }

        todo!()
    }

    fn put_u8p3<const A: u8, const B: u8, const C: u8>(&mut self, a: u8, b: u8, c: u8) {
        let _ = c;
        let _ = b;
        let _ = a;
        const {
            assert!(A + B + C == 8);
        }

        todo!()
    }

    fn put_u8p4<const A: u8, const B: u8, const C: u8, const D: u8>(
        &mut self,
        a: u8,
        b: u8,
        c: u8,
        d: u8,
    ) {
        let _ = d;
        let _ = c;
        let _ = b;
        let _ = a;
        const {
            assert!(A + B + C + D == 8);
        }

        todo!()
    }

    fn put_u16p2<const A: u8, const B: u8>(&mut self, a: u16, b: u16) {
        let _ = b;
        let _ = a;
        const {
            assert!(A + B == 16);
        }

        todo!()
    }

    fn put_u16p3<const A: u8, const B: u8, const C: u8>(&mut self, a: u16, b: u16, c: u16) {
        let _ = c;
        let _ = b;
        let _ = a;
        const {
            assert!(A + B + C == 16);
        }

        todo!()
    }

    fn put_u16p4<const A: u8, const B: u8, const C: u8, const D: u8>(
        &mut self,
        a: u16,
        b: u16,
        c: u16,
        d: u16,
    ) {
        let _ = d;
        let _ = c;
        let _ = b;
        let _ = a;
        const {
            assert!(A + B + C + D == 16);
        }

        todo!()
    }
}

impl<T: Writer> WriterExt for T {}

pub trait Decoder<T> {
    type Error;

    fn decode<R: Reader>(&mut self, reader: &mut R) -> Result<T, Self::Error>;
}

pub trait Encoder<T> {
    type Error;

    fn encode<W: Writer>(&mut self, item: &T, writer: &mut W) -> Result<(), Self::Error>;
    fn size_hint() -> (usize, Option<usize>) {
        (0, None)
    }
    fn estimate_size(&self, _: &T) -> usize {
        Self::size_hint().0
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct BytesDecoder;

impl Decoder<Bytes> for BytesDecoder {
    type Error = crate::Void;

    fn decode<R: Reader>(&mut self, reader: &mut R) -> Result<Bytes, Self::Error> {
        Ok(reader.read_bytes(reader.remaining()).unwrap())
    }
}
