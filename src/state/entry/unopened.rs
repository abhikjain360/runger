use std::path::PathBuf;
use std::sync::Arc;

#[cfg_attr(debug_assertions, derive(Debug))]
pub(crate) struct Unopened {
    pub(crate) select_on_open: Option<Arc<PathBuf>>,
}
