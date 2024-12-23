use std::path::PathBuf;
use std::sync::Arc;

use crate::Path;

#[cfg_attr(debug_assertions, derive(Debug))]
pub(crate) struct Unopened {
    pub(crate) select_on_open: Option<Path>,
}
