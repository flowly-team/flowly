pub trait Error: std::error::Error + Send + Sync + 'static {}

impl<T: std::error::Error + Send + Sync + 'static> Error for T {}
