use std::{
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
    sync::Arc,
};

use bytes::Bytes;
use flowly_core::{DataFrame, FrameSource, MemBlock};
use flowly_service::{Context, Service};
use glob::MatchOptions;
use tokio::io::AsyncReadExt;

use crate::error::Error;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct FileSouce {
    path: String,
}

impl FrameSource for FileSouce {
    type Source = flowly_core::Void;

    fn source(&self) -> &Self::Source {
        unreachable!()
    }

    fn kind(&self) -> flowly_core::FrameSourceKind {
        flowly_core::FrameSourceKind::File
    }

    fn url(&self) -> &str {
        &self.path
    }

    fn name(&self) -> &str {
        &self.path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WithSource<E> {
    pub inner: E,
    pub source: Arc<FileSouce>,
}

impl<E> WithSource<E> {
    pub fn new(inner: E, source: Arc<FileSouce>) -> Self {
        Self { inner, source }
    }
}

impl<E> Deref for WithSource<E> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl<E> DerefMut for WithSource<E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<U: ?Sized, E: AsRef<U>> AsRef<U> for WithSource<E> {
    fn as_ref(&self) -> &U {
        self.inner.as_ref()
    }
}

impl<E> DataFrame for WithSource<E>
where
    E: MemBlock + Clone,
{
    type Source = Arc<FileSouce>;
    type Chunk = E;

    fn source(&self) -> &Self::Source {
        &self.source
    }

    fn chunks(&self) -> impl Send + Iterator<Item = <Self::Chunk as MemBlock>::Ref<'_>> {
        std::iter::once(self.inner.borrow())
    }

    fn into_chunks(self) -> impl Send + Iterator<Item = Self::Chunk> {
        std::iter::once(self.inner)
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

impl<P: AsRef<Path> + Send> Service<P> for DirReader {
    type Out = Result<WithSource<PathBuf>, Error>;

    fn handle(&mut self, dir: P, cx: &Context) -> impl futures::Stream<Item = Self::Out> + Send {
        async_stream::stream! {
            let pattern = format!("{}/{}", dir.as_ref().display().to_string().trim_end_matches('/'), self.pattern);
            let shared = Arc::new(FileSouce { path: pattern });

            match glob::glob_with(&shared.path, self.options) {
                Ok(paths) => {
                    for p in paths {
                        match cx.abort_recv.has_changed() {
                            Ok(true) | Err(_) => break,
                            _ => ()
                        }

                        yield p.map(|inner| WithSource::new(inner , shared.clone())).map_err(Into::into);
                    }
                }

                Err(err) => yield Err(err.into()),
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

impl<P: AsRef<Path> + Send + Sync> Service<P> for FileReader {
    type Out = Result<WithSource<Bytes>, Error>;

    fn handle(&mut self, path: P, cx: &Context) -> impl futures::Stream<Item = Self::Out> + Send {
        async_stream::stream! {
            let mut buf = vec![0u8; self.chunk_size];

            match tokio::fs::File::open(&path).await {
                Ok(mut file) => {
                    let shared = Arc::new(FileSouce { path: path.as_ref().display().to_string() });

                    loop {
                        match cx.abort_recv.has_changed() {
                            Ok(true) | Err(_) => break,
                            _ => ()
                        }

                        yield match file.read(&mut buf[..]).await {
                            Ok(0) => break,
                            Ok(n) => Ok(WithSource::new(buf[0..n].to_vec().into(), shared.clone())),
                            Err(err) => Err(err.into())
                        };
                    }
                },
                Err(err) => yield Err(err.into()),
            }
        }
    }
}
