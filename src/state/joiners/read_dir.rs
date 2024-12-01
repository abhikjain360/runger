use std::io;
use std::path::PathBuf;
use std::sync::Arc;

use futures::future::{BoxFuture, FutureExt};
use futures::stream::FuturesUnordered;
use tokio_stream::wrappers::ReadDirStream;
use tokio_stream::StreamExt;

#[cfg_attr(debug_assertions, derive(Debug))]
pub(crate) struct ReadDirJoiner {
    #[allow(clippy::type_complexity)]
    inner: FuturesUnordered<BoxFuture<'static, io::Result<(Arc<PathBuf>, Vec<Arc<PathBuf>>)>>>,
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
                let read_dir = tokio::fs::read_dir(path.as_ref()).await?;
                let stream = ReadDirStream::new(read_dir);

                let entries = stream
                    .map(|entry_res| Ok(Arc::new(entry_res?.path().to_path_buf())))
                    .collect::<io::Result<Vec<_>>>()
                    .await?;

                Ok((path, entries))
            }
            .boxed(),
        );
    }

    pub(crate) async fn join_next(
        &mut self,
    ) -> Option<io::Result<(Arc<PathBuf>, Vec<Arc<PathBuf>>)>> {
        futures::StreamExt::next(&mut self.inner).await
    }
}
