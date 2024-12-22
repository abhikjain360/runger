use std::io;
use std::path::PathBuf;
use std::sync::Arc;

use futures::future::BoxFuture;
use futures::stream::FuturesUnordered;
use futures::FutureExt;

#[cfg_attr(debug_assertions, derive(Debug))]
pub(crate) struct DeleteJoiner {
    // TODO: remove boxed
    inner: FuturesUnordered<BoxFuture<'static, io::Result<Arc<PathBuf>>>>,
}

impl DeleteJoiner {
    pub(crate) fn new() -> Self {
        Self {
            inner: FuturesUnordered::new(),
        }
    }

    /// Returns true if the set is empty.
    pub(crate) fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn spawn(&mut self, path: Arc<PathBuf>) {
        self.inner.push(
            async move {
                if !path.exists() {
                    return Err(io::Error::new(
                        io::ErrorKind::NotFound,
                        format!("{path:?} does not exist"),
                    ));
                }

                if path.is_dir() {
                    tokio::fs::remove_dir_all(path.as_ref()).await?;
                } else if path.is_file() {
                    tokio::fs::remove_file(path.as_ref()).await?;
                }

                Ok(path)
            }
            .boxed(),
        );
    }

    pub(crate) async fn join_next(&mut self) -> Option<io::Result<Arc<PathBuf>>> {
        futures::StreamExt::next(&mut self.inner).await
    }
}
