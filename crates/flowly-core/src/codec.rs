pub use bytes::TryGetError;
use bytes::{Buf, BufMut, Bytes, BytesMut};

pub trait Reader {
    fn remaining(&self) -> usize;

    fn advance(&mut self, cnt: usize);
    fn read_f64(&mut self) -> Result<f64, TryGetError>;
    fn read_f32(&mut self) -> Result<f32, TryGetError>;
    fn read_int(&mut self, nbytes: usize) -> Result<i64, TryGetError>;
    fn read_uint(&mut self, nbytes: usize) -> Result<u64, TryGetError>;
    fn read_i64(&mut self) -> Result<i64, TryGetError>;
    fn read_u64(&mut self) -> Result<u64, TryGetError>;
    fn read_i32(&mut self) -> Result<i32, TryGetError>;
    fn read_u32(&mut self) -> Result<u32, TryGetError>;
    fn read_i16(&mut self) -> Result<i16, TryGetError>;
    fn read_u16(&mut self) -> Result<u16, TryGetError>;
    fn read_i8(&mut self) -> Result<i8, TryGetError>;
    fn read_u8(&mut self) -> Result<u8, TryGetError>;

    fn read_array<const N: usize>(&mut self) -> Result<[u8; N], TryGetError>;
    fn read_bytes(&mut self, len: usize) -> Result<Bytes, TryGetError>;
    fn read_string(&mut self, len: usize) -> Result<String, TryGetError>;

    #[inline]
    fn has_remaining(&self) -> bool {
        self.remaining() > 0
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
        Ok(self.read_int(6)? as i64)
    }

    #[inline]
    fn read_u48(&mut self) -> Result<u64, TryGetError> {
        Ok(self.read_uint(6)? as u64)
    }

    #[inline]
    fn read_remaining(&mut self) -> Bytes {
        self.read_bytes(self.remaining()).unwrap()
    }
}

impl<T: Buf> Reader for T {
    #[inline]
    fn remaining(&self) -> usize {
        Buf::remaining(self)
    }

    #[inline]
    fn advance(&mut self, cnt: usize) {
        Buf::advance(self, cnt)
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
    fn read_string(&mut self, len: usize) -> Result<String, TryGetError> {
        let mut buf = BytesMut::with_capacity(len);
        buf.put(self.take(len));

        String::from_utf8(buf.into()).map_err(|_| TryGetError {
            requested: 0,
            available: 0,
        })
    }

    #[inline]
    fn read_bytes(&mut self, len: usize) -> Result<Bytes, TryGetError> {
        let available = Buf::remaining(self);
        if available < len {
            Err(TryGetError {
                requested: len,
                available,
            })
        } else {
            Ok(self.copy_to_bytes(len))
        }
    }

    #[inline]
    fn read_array<const N: usize>(&mut self) -> Result<[u8; N], TryGetError> {
        let mut data = [0u8; N];
        self.try_copy_to_slice(&mut data)?;
        Ok(data)
    }
}

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

pub trait Writer {
    fn remaining(&self) -> usize;
    fn put_f64(&mut self, val: f64);
    fn put_f32(&mut self, val: f32);
    fn put_int(&mut self, val: i64, nbytes: usize);
    fn put_uint(&mut self, val: u64, nbytes: usize);
    fn put_i64(&mut self, val: i64);
    fn put_u64(&mut self, val: u64);
    fn put_i32(&mut self, val: i32);
    fn put_u32(&mut self, val: u32);
    fn put_i16(&mut self, val: i16);
    fn put_u16(&mut self, val: u16);
    fn put_i8(&mut self, val: i8);
    fn put_u8(&mut self, val: u8);
    fn put_slice(&mut self, slice: &[u8]);

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

impl<T: BufMut> Writer for T {
    #[inline]
    fn remaining(&self) -> usize {
        self.remaining_mut()
    }

    #[inline]
    fn put_f64(&mut self, val: f64) {
        self.put_f64(val);
    }

    #[inline]
    fn put_f32(&mut self, val: f32) {
        self.put_f32(val);
    }

    #[inline]
    fn put_int(&mut self, val: i64, nbytes: usize) {
        self.put_int(val, nbytes);
    }

    #[inline]
    fn put_uint(&mut self, val: u64, nbytes: usize) {
        self.put_uint(val, nbytes);
    }

    #[inline]
    fn put_i64(&mut self, val: i64) {
        self.put_i64(val);
    }

    #[inline]
    fn put_u64(&mut self, val: u64) {
        self.put_u64(val);
    }

    #[inline]
    fn put_i32(&mut self, val: i32) {
        self.put_i32(val);
    }

    #[inline]
    fn put_u32(&mut self, val: u32) {
        self.put_u32(val);
    }

    #[inline]
    fn put_i16(&mut self, val: i16) {
        self.put_i16(val);
    }

    #[inline]
    fn put_u16(&mut self, val: u16) {
        self.put_u16(val);
    }

    #[inline]
    fn put_i8(&mut self, val: i8) {
        self.put_i8(val);
    }

    #[inline]
    fn put_u8(&mut self, val: u8) {
        self.put_u8(val);
    }

    #[inline]
    fn put_slice(&mut self, slice: &[u8]) {
        self.put_slice(slice);
    }
}

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
    fn estimate_size(&self, item: &T) -> usize {
        let _ = item;
        0
    }
}
