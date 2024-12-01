use std::io;
use std::path::PathBuf;
use std::sync::Arc;

use futures::future::{BoxFuture, FutureExt};
use futures::stream::FuturesUnordered;
use tokio_stream::wrappers::ReadDirStream;
use tokio_stream::StreamExt;

#[cfg_attr(debug_assertions, derive(Debug))]
pub(crate) struct ReadDirJoiner {
    inner: FuturesUnordered<BoxFuture<'static, ReadDirResult>>,
}

pub(crate) struct ReadDirResult {
    pub(crate) path: Arc<PathBuf>,
    pub(crate) kind: ReadDirResultKind,
}

pub(crate) enum ReadDirResultKind {
    Ok(Vec<Arc<PathBuf>>),
    Err(io::Error),
    PermissionDenied,
}

impl ReadDirJoiner {
    pub(crate) fn new() -> Self {
        Self {
            inner: FuturesUnordered::new(),
        }
    }

    /// Returns true if the set is empty.
    pub(crate) fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub(crate) fn spawn(&mut self, path: Arc<PathBuf>) {
        self.inner.push(
            async move {
                let read_dir_result = tokio::fs::read_dir(path.as_ref()).await;
                let read_dir = match read_dir_result {
                    Ok(read_dir) => read_dir,
                    Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
                        return ReadDirResult::permission_denied(path);
                    }
                    Err(e) => return ReadDirResult::err(path, e),
                };
                let stream = ReadDirStream::new(read_dir);

                let entries_result = stream
                    .map(|entry_res| {
                        Ok(Arc::new(
                            entry_res
                                .inspect_err(|e| {
                                    tracing::error!("unable to read dir entry {:?}: {e}", path)
                                })?
                                .path()
                                .to_path_buf(),
                        ))
                    })
                    .collect::<io::Result<Vec<_>>>()
                    .await;

                let entries = match entries_result {
                    Ok(entries) => entries,
                    Err(e) => return ReadDirResult::err(path, e),
                };

                ReadDirResult::ok(path, entries)
            }
            .boxed(),
        );
    }

    pub(crate) async fn join_next(&mut self) -> Option<ReadDirResult> {
        futures::StreamExt::next(&mut self.inner).await
    }
}

impl ReadDirResult {
    fn permission_denied(path: Arc<PathBuf>) -> Self {
        Self {
            path,
            kind: ReadDirResultKind::PermissionDenied,
        }
    }

    pub(crate) fn ok(path: Arc<PathBuf>, entries: Vec<Arc<PathBuf>>) -> Self {
        Self {
            path,
            kind: ReadDirResultKind::Ok(entries),
        }
    }

    pub(crate) fn err(path: Arc<PathBuf>, err: io::Error) -> Self {
        Self {
            path,
            kind: ReadDirResultKind::Err(err),
        }
    }
}
