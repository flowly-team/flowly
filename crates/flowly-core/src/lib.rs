mod codec;
mod frame;
mod memory;

pub use codec::Fourcc;
pub use frame::{Frame, FrameFlags};
pub use memory::{CpuAllocator, CpuMemBlock, Error, MemBlock, MemoryAllocator};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Void {}
impl std::fmt::Display for Void {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unreachable!()
    }
}
impl std::error::Error for Void {}
