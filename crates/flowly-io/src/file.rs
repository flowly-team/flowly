use std::{
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
    pin::pin,
    sync::Arc,
};

use bytes::Bytes;
use flowly_service::Service;
use futures::StreamExt;
use glob::MatchOptions;
use tokio::io::AsyncReadExt;

use crate::error::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spread<E, S> {
    pub inner: E,
    pub shared: Arc<S>,
}

impl<E, S> Spread<E, S> {
    pub fn new(inner: E, shared: Arc<S>) -> Self {
        Self { inner, shared }
    }
}

impl<E, S> Deref for Spread<E, S> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl<E, S> DerefMut for Spread<E, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<U: ?Sized, E: AsRef<U>, S> AsRef<U> for Spread<E, S> {
    fn as_ref(&self) -> &U {
        self.inner.as_ref()
    }
}

#[derive(Debug, Clone)]
pub struct DirReader {
    pattern: String,
    options: MatchOptions,
}

impl DirReader {
    pub fn new(pattern: String, options: MatchOptions) -> Self {
        Self { pattern, options }
    }
}

impl<P, E> Service<Result<P, E>> for DirReader
where
    P: AsRef<Path> + Send + Sync,
    E: std::error::Error + Send + Sync + 'static,
{
    type Out = Result<Spread<PathBuf, P>, Error<E>>;

    fn handle(
        self,
        input: impl futures::Stream<Item = Result<P, E>> + Send,
    ) -> impl futures::Stream<Item = Self::Out> + Send {
        async_stream::stream! {
            let mut stream = pin!(input);

            while let Some(res) = stream.next().await {
                match res {
                    Ok(dir) => {
                        let pattern = format!("{}/{}", dir.as_ref().display().to_string().trim_end_matches('/'), self.pattern);
                        let shared = Arc::new(dir);

                        match glob::glob_with(&pattern, self.options) {
                            Ok(paths) => {
                                for p in paths {
                                    yield p.map(|inner| Spread::new(inner , shared.clone())).map_err(Into::into);
                                }
                            }

                            Err(err) => yield Err(err.into()),
                        }
                    }

                    Err(err) => yield Err(Error::Other(err)),
                }

            }

        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FileReader {
    chunk_size: usize,
}

impl FileReader {
    pub fn new(chunk_size: usize) -> Self {
        Self { chunk_size }
    }
}

impl Default for FileReader {
    fn default() -> Self {
        Self { chunk_size: 8192 }
    }
}

impl<P: AsRef<Path> + Send + Sync, E: std::error::Error + Send + Sync + 'static>
    Service<Result<P, E>> for FileReader
{
    type Out = Result<Spread<Bytes, P>, Error<E>>;

    fn handle(
        self,
        input: impl futures::Stream<Item = Result<P, E>> + Send,
    ) -> impl futures::Stream<Item = Self::Out> + Send {
        async_stream::stream! {
            let mut input = pin!(input);
            let mut buf = vec![0u8; self.chunk_size];

            while let Some(res) = input.next().await  {
                match res {
                    Ok(path) => {
                        println!("path {}", path.as_ref().display());
                        match tokio::fs::File::open(&path).await {
                            Ok(mut file) => {
                                let shared = Arc::new(path);
                                loop {
                                    yield match file.read(&mut buf[..]).await {
                                        Ok(0) => break,
                                        Ok(n) => Ok(Spread::new(buf[0..n].to_vec().into(), shared.clone())),
                                        Err(err) => Err(err.into())
                                    };
                                }
                            },
                            Err(err) => yield Err(err.into()),
                        }
                    }

                    Err(err) => yield Err(Error::Other(err)),
                }
            }
        }
    }
}
