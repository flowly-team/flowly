use std::alloc::Layout;

use bytes::Bytes;

#[derive(Debug, thiserror::Error)]
pub enum MemError {
    #[error("Mem error")]
    CpuAllocationError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemDevice {
    Cpu,
    Gpu(u32),
}

pub trait MemBlock: Send {
    type Ref<'a>: MemBlock + Clone
    where
        Self: 'a;

    fn device(&self) -> MemDevice;
    fn borrow(&self) -> Self::Ref<'_>;
    fn map_to_cpu(&self) -> &[u8];

    // Generic non-optimal default implementations
    #[inline]
    fn len(&self) -> usize {
        self.map_to_cpu().len()
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    fn layout(&self) -> Layout {
        let mapped = self.map_to_cpu();
        let len = mapped.len();
        let ptr = self.map_to_cpu() as *const [u8] as *const u8 as usize;

        unsafe { Layout::from_size_align_unchecked(len, 1 << ptr.trailing_zeros()) }
    }

    #[inline]
    fn into_cpu_bytes(self) -> Bytes
    where
        Self: Sized,
    {
        Bytes::copy_from_slice(self.map_to_cpu())
    }
}

impl MemBlock for Bytes {
    type Ref<'a> = &'a Bytes;

    #[inline]
    fn borrow(&self) -> Self::Ref<'_> {
        self
    }

    #[inline]
    fn device(&self) -> MemDevice {
        MemDevice::Cpu
    }

    #[inline]
    fn map_to_cpu(&self) -> &[u8] {
        self
    }

    #[inline]
    fn into_cpu_bytes(self) -> Bytes {
        self
    }
}

impl MemBlock for Vec<u8> {
    type Ref<'a> = &'a [u8];

    #[inline]
    fn borrow(&self) -> Self::Ref<'_> {
        self
    }

    #[inline]
    fn device(&self) -> MemDevice {
        MemDevice::Cpu
    }

    #[inline]
    fn map_to_cpu(&self) -> &[u8] {
        self
    }

    #[inline]
    fn into_cpu_bytes(self) -> Bytes {
        self.into()
    }
}

impl MemBlock for &[u8] {
    type Ref<'b>
        = &'b [u8]
    where
        Self: 'b;

    #[inline]
    fn borrow(&self) -> Self::Ref<'_> {
        self
    }

    #[inline]
    fn device(&self) -> MemDevice {
        MemDevice::Cpu
    }

    #[inline]
    fn map_to_cpu(&self) -> &[u8] {
        self
    }
}

impl MemBlock for &Bytes {
    type Ref<'b>
        = &'b [u8]
    where
        Self: 'b;

    #[inline]
    fn borrow(&self) -> Self::Ref<'_> {
        self
    }

    #[inline]
    fn device(&self) -> MemDevice {
        MemDevice::Cpu
    }

    #[inline]
    fn map_to_cpu(&self) -> &[u8] {
        self
    }
}

pub trait MemAlloc {
    type Data: MemBlock;
    type Error: std::error::Error + Send + Sync + 'static;

    fn device(&self) -> MemDevice;
    fn alloc(&self, data: &[u8]) -> Result<Self::Data, Self::Error>;
}

impl<A: MemAlloc> MemAlloc for std::sync::Arc<A> {
    type Data = A::Data;
    type Error = A::Error;

    fn device(&self) -> MemDevice {
        (**self).device()
    }

    fn alloc(&self, data: &[u8]) -> Result<Self::Data, Self::Error> {
        (**self).alloc(data)
    }
}

#[derive(Debug, Clone)]
pub struct CpuAllocator;
impl MemAlloc for CpuAllocator {
    type Data = Bytes;
    type Error = MemError;

    fn device(&self) -> MemDevice {
        MemDevice::Cpu
    }

    fn alloc(&self, data: &[u8]) -> Result<Self::Data, Self::Error> {
        Ok(Bytes::copy_from_slice(data))
    }
}
