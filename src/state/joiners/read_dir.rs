use std::{io, path::PathBuf, sync::Arc};

type ReadDirJoinerInner = tokio::task::JoinSet<io::Result<(Arc<PathBuf>, tokio::fs::ReadDir)>>;

pub(crate) struct ReadDirJoiner {
    inner: ReadDirJoinerInner,
}

impl ReadDirJoiner {
    pub(crate) fn new() -> Self {
        Self {
            inner: ReadDirJoinerInner::new(),
        }
    }

    /// Returns true if the set is empty.
    pub(crate) fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub(crate) fn spawn(&mut self, path: Arc<PathBuf>) {
        self.inner.spawn(async move {
            tokio::fs::read_dir(path.as_ref())
                .await
                .map(|read_dir| (path, read_dir))
        });
    }
}
