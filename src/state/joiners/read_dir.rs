use std::collections::VecDeque;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;

use futures::future::{BoxFuture, FutureExt};

pub(crate) struct ReadDirJoiner {
    // TODO: remove boxed
    inner: VecDeque<BoxFuture<'static, ReadDirResult>>,
}

pub(crate) struct ReadDirResult {
    pub(crate) path: Arc<PathBuf>,
    pub(crate) kind: ReadDirResultKind,
}

pub(crate) enum ReadDirResultKind {
    Ok(Vec<Arc<PathBuf>>),
    Err(io::Error),
    PermissionDenied,
    NotADirectory,
}

impl ReadDirJoiner {
    pub(crate) fn new() -> Self {
        Self {
            inner: VecDeque::new(),
        }
    }

    /// Returns true if the set is empty.
    pub(crate) fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn spawn(&mut self, path: Arc<PathBuf>) {
        self.inner.push_front(
            async move {
                let read_dir_result = tokio::fs::read_dir(path.as_ref()).await;
                let mut read_dir = match read_dir_result {
                    Ok(read_dir) => read_dir,
                    Err(e) => match e.kind() {
                        io::ErrorKind::PermissionDenied => {
                            return ReadDirResult::permission_denied(path);
                        }
                        io::ErrorKind::NotADirectory => {
                            return ReadDirResult::not_a_directory(path);
                        }
                        _ => return ReadDirResult::err(path, e),
                    },
                };

                let mut entries = vec![];

                while let Some(dir_entry) = match read_dir.next_entry().await {
                    Ok(dir_entry) => dir_entry,
                    Err(e) => {
                        return ReadDirResult::err(path, e);
                    }
                } {
                    let path = Arc::new(dir_entry.path().to_path_buf());
                    entries.push(path.clone());
                }

                ReadDirResult::ok(path, entries)
            }
            .boxed(),
        );
    }

    pub(crate) async fn join_next(&mut self) -> Option<ReadDirResult> {
        let first = self.inner.front_mut()?;
        let ret = first.await;
        self.inner.pop_front();
        Some(ret)
    }
}

impl ReadDirResult {
    fn permission_denied(path: Arc<PathBuf>) -> Self {
        Self {
            path,
            kind: ReadDirResultKind::PermissionDenied,
        }
    }

    fn not_a_directory(path: Arc<PathBuf>) -> Self {
        Self {
            path,
            kind: ReadDirResultKind::NotADirectory,
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
