use bytes::Bytes;

#[derive(Debug, thiserror::Error)]
pub enum MemError {
    #[error("Mem error")]
    CpuAllocationError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemType {
    Cpu,
    Gpu(u32),
}

pub trait MemBlock {
    fn to_cpu_bytes(&self) -> Bytes;
}

pub trait MemAlloc {
    type Data: MemBlock;
    type Error: std::error::Error + Send + Sync + 'static;

    fn memory_type(&self) -> MemType;
    fn alloc_frame(&self, data: &[u8]) -> Result<Self::Data, Self::Error>;
}

impl<A: MemAlloc> MemAlloc for std::sync::Arc<A> {
    type Data = A::Data;
    type Error = A::Error;

    fn memory_type(&self) -> MemType {
        (**self).memory_type()
    }

    fn alloc_frame(&self, data: &[u8]) -> Result<Self::Data, Self::Error> {
        (**self).alloc_frame(data)
    }
}

pub struct CpuMemBlock(Bytes);
impl MemBlock for CpuMemBlock {
    fn to_cpu_bytes(&self) -> Bytes {
        self.0.clone()
    }
}

pub struct CpuAllocator;

impl MemAlloc for CpuAllocator {
    type Data = CpuMemBlock;
    type Error = MemError;

    fn memory_type(&self) -> MemType {
        MemType::Cpu
    }

    fn alloc_frame(&self, data: &[u8]) -> Result<Self::Data, Self::Error> {
        Ok(CpuMemBlock(Bytes::from_owner(data.to_vec())))
    }
}
