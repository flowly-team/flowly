use crate::FrameSource;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Void {}

impl Default for Void {
    fn default() -> Self {
        unreachable!()
    }
}

impl std::fmt::Display for Void {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unreachable!()
    }
}

impl std::error::Error for Void {}
impl FrameSource for Void {
    type Source = Void;

    fn kind(&self) -> crate::FrameSourceKind {
        unreachable!()
    }

    fn url(&self) -> &str {
        unreachable!()
    }

    fn source(&self) -> &Self::Source {
        unreachable!()
    }

    fn name(&self) -> &str {
        unreachable!()
    }
}
