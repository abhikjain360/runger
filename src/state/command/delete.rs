use std::path::PathBuf;
use std::sync::Arc;

// TODO: support deleting multiple entries
#[cfg_attr(debug_assertions, derive(Debug))]
pub(crate) enum DeleteCommand {
    Init,
    Typing {
        input: String,
    },
    #[expect(dead_code)]
    Confirmed {
        path: Arc<PathBuf>,
    },
}
