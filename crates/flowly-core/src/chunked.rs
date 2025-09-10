use std::{
    collections::{VecDeque, vec_deque},
    io::IoSlice,
};

use bytes::{Buf, Bytes};

#[derive(Debug, Default, Clone)]
pub struct Chunked<T> {
    remaining: usize,
    chunks: VecDeque<T>,
}

impl<T> Chunked<T> {
    pub fn new() -> Self {
        Self {
            remaining: 0,
            chunks: Default::default(),
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            remaining: 0,
            chunks: VecDeque::with_capacity(cap),
        }
    }

    #[inline]
    pub fn iter(&self) -> vec_deque::Iter<'_, T> {
        self.chunks.iter()
    }
}

impl<T: Buf> Chunked<T> {
    #[inline]
    pub fn put(&mut self, chunk: T) {
        if !chunk.has_remaining() {
            return;
        }

        self.remaining += chunk.remaining();
        self.chunks.push_back(chunk);
    }
}

impl<T: Buf> Buf for Chunked<T> {
    #[inline]
    fn remaining(&self) -> usize {
        self.remaining
    }

    #[inline]
    fn chunk(&self) -> &[u8] {
        self.chunks.front().map(|x| x.chunk()).unwrap_or(&[])
    }

    fn advance(&mut self, mut cnt: usize) {
        self.remaining -= cnt;

        while cnt > 0 {
            let Some(chunk) = self.chunks.front_mut() else {
                panic!("advance: no available chunks!");
            };

            let len = chunk.remaining();
            if cnt < len {
                chunk.advance(cnt);
                return;
            } else {
                cnt -= len;
                self.chunks.pop_front();
            }
        }
    }

    fn chunks_vectored<'a>(&'a self, dst: &mut [IoSlice<'a>]) -> usize {
        let mut iter = self.chunks.iter();
        let mut len = 0;

        while len < dst.len() {
            let Some(chunk) = iter.next() else {
                break;
            };

            len += chunk.chunks_vectored(&mut dst[len..]);
        }

        len
    }

    fn copy_to_bytes(&mut self, len: usize) -> Bytes {
        if let Some(chunk) = self.chunks.front_mut() {
            if chunk.remaining() > len {
                self.remaining -= len;
                chunk.copy_to_bytes(len)
            } else if chunk.remaining() == len {
                self.remaining -= len;
                self.chunks.pop_front().unwrap().copy_to_bytes(len)
            } else {
                use bytes::buf::BufMut;
                let mut ret = bytes::BytesMut::with_capacity(len);
                ret.put(self.take(len));
                ret.freeze()
            }
        } else {
            panic!("copy_to_bytes: no available chunks!");
        }
    }
}

impl<T: Buf> From<Vec<T>> for Chunked<T> {
    fn from(value: Vec<T>) -> Self {
        Self {
            remaining: value.iter().map(|x| x.remaining()).sum(),
            chunks: value.into(),
        }
    }
}

impl<T> IntoIterator for Chunked<T> {
    type Item = T;
    type IntoIter = vec_deque::IntoIter<T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.chunks.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, IoSlice, Write};

    use bytes::{Buf, Bytes};

    use super::Chunked;

    #[test]
    fn test_chunked_bytes_copy_to_slice() {
        let mut bytes = Chunked::new();
        bytes.put(Bytes::from_static(b"hello"));
        bytes.put(Bytes::from_static(b", "));
        bytes.put(Bytes::from_static(b"world"));

        assert_eq!(bytes.remaining(), 12);

        let slice = &mut [0u8; 12];
        bytes.copy_to_slice(slice);

        assert_eq!(bytes.remaining(), 0);
        assert_eq!(slice, b"hello, world");

        let mut bytes = Chunked::new();
        bytes.put(Bytes::from_static(b"hello, world"));

        assert_eq!(bytes.remaining(), 12);

        let slice = &mut [0u8; 5];
        bytes.copy_to_slice(slice);

        assert_eq!(bytes.remaining(), 7);
        assert_eq!(slice, b"hello");
        assert_eq!(
            bytes.chunks.front().map(|x| x.chunk()),
            Some(&b", world"[..])
        );
    }

    #[test]
    fn test_chunked_bytes_copy_to_bytes() {
        let mut bytes = Chunked::new();
        bytes.put(Bytes::from_static(b"hello"));
        bytes.put(Bytes::from_static(b", "));
        bytes.put(Bytes::from_static(b"world"));

        assert_eq!(bytes.remaining(), 12);

        let copy = bytes.copy_to_bytes(12);

        assert_eq!(bytes.remaining(), 0);
        assert_eq!(copy.as_ref(), b"hello, world");

        let mut bytes = Chunked::new();
        bytes.put(Bytes::from_static(b"hello, world"));

        assert_eq!(bytes.remaining(), 12);

        let copy = bytes.copy_to_bytes(5);

        assert_eq!(bytes.remaining(), 7);
        assert_eq!(copy.as_ref(), b"hello");

        let mut bytes = Chunked::new();
        bytes.put(Bytes::from_static(b"hello, world"));

        assert_eq!(bytes.remaining(), 12);

        let copy = bytes.copy_to_bytes(12);

        assert_eq!(bytes.remaining(), 0);
        assert_eq!(copy.as_ref(), b"hello, world");
    }

    #[test]
    fn test_chunken_bytes_data() {
        let mut bytes = Chunked::new();
        bytes.put(Bytes::from_static(&[0]));
        bytes.put(Bytes::from_static(&[0]));
        bytes.put(Bytes::from_static(&[0]));
        bytes.put(Bytes::from_static(&[12]));
        bytes.put(Bytes::from_static(&[0]));

        assert_eq!(bytes.get_u32(), 12);
        assert_eq!(bytes.remaining(), 1);

        let mut bytes = Chunked::new();
        bytes.put(Bytes::from_static(&[0, 0]));
        bytes.put(Bytes::from_static(&[0, 12]));
        bytes.put(Bytes::from_static(&[0]));

        assert_eq!(bytes.get_u32(), 12);
        assert_eq!(bytes.remaining(), 1);

        let mut bytes = Chunked::new();
        bytes.put(Bytes::from_static(&[0, 0, 0, 12]));
        bytes.put(Bytes::from_static(&[0]));

        assert_eq!(bytes.get_u32(), 12);
        assert_eq!(bytes.remaining(), 1);

        let mut bytes = Chunked::new();
        bytes.put(Bytes::from_static(&[0, 0, 0, 12, 0]));

        assert_eq!(bytes.get_u32(), 12);
        assert_eq!(bytes.remaining(), 1);

        let mut bytes = Chunked::new();
        bytes.put(Bytes::from_static(&[0, 0, 0, 12]));

        assert_eq!(bytes.get_u32(), 12);
        assert_eq!(bytes.remaining(), 0);
    }

    #[test]
    fn test_chunked_bytes_vectored() {
        let mut dst = Vec::new();

        let mut bytes = Chunked::new();
        bytes.put(Bytes::from_static(b"hello"));
        bytes.put(Bytes::from_static(b", "));
        bytes.put(Bytes::from_static(b"world"));
        bytes.put(Bytes::from_static(b"!"));

        assert_eq!(bytes.remaining(), 13);

        let mut bufs = [
            IoSlice::new(&[]),
            IoSlice::new(&[]),
            IoSlice::new(&[]),
            IoSlice::new(&[]),
            IoSlice::new(&[]),
            IoSlice::new(&[]),
            IoSlice::new(&[]),
            IoSlice::new(&[]),
        ];

        assert_eq!(bytes.chunks_vectored(&mut bufs), 4);
        let _ = Cursor::new(&mut dst).write_vectored(&bufs[0..4]).unwrap();

        assert_eq!(&dst[..], b"hello, world!");

        let mut dst = Vec::new();
        let mut bytes = Chunked::new();
        bytes.put(Bytes::from_static(b"hello"));
        bytes.put(Bytes::from_static(b", "));
        bytes.put(Bytes::from_static(b"world"));
        bytes.put(Bytes::from_static(b"!"));

        assert_eq!(bytes.remaining(), 13);

        let mut bufs = [IoSlice::new(&[]), IoSlice::new(&[])];

        assert_eq!(bytes.chunks_vectored(&mut bufs), 2);
        let _ = Cursor::new(&mut dst).write_vectored(&bufs[0..2]).unwrap();

        assert_eq!(&dst[..], b"hello, ");
    }
}
