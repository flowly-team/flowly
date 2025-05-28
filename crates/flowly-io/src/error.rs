#[derive(Debug)]
pub enum Void {}
impl std::fmt::Display for Void {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unreachable!()
    }
}
impl std::error::Error for Void {}

#[derive(Debug, thiserror::Error)]
pub enum Error<E: std::error::Error + Send + Sync + 'static = Void> {
    #[error("Io Error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Glob Error: {0}")]
    GlobError(#[from] glob::GlobError),

    #[error("Glob Pattern Error: {0}")]
    GlobPatternError(#[from] glob::PatternError),

    #[error(transparent)]
    Other(E),
}
