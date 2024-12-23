use std::borrow::Borrow;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Path(Arc<PathBuf>);

impl Path {
    pub(crate) fn new(path: Arc<PathBuf>) -> Self {
        Self(path)
    }
}

impl Borrow<PathBuf> for Path {
    fn borrow(&self) -> &PathBuf {
        self.0.as_ref()
    }
}

impl Borrow<std::path::Path> for Path {
    fn borrow(&self) -> &std::path::Path {
        self.0.as_path()
    }
}

impl Borrow<Arc<PathBuf>> for Path {
    fn borrow(&self) -> &Arc<PathBuf> {
        &self.0
    }
}

impl AsRef<PathBuf> for Path {
    fn as_ref(&self) -> &PathBuf {
        &self.0
    }
}

impl From<PathBuf> for Path {
    fn from(path: PathBuf) -> Self {
        Self::new(Arc::new(path))
    }
}

impl From<Arc<PathBuf>> for Path {
    fn from(path: Arc<PathBuf>) -> Self {
        Self::new(path)
    }
}

impl Deref for Path {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}
