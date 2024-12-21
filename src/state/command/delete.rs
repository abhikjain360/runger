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

pub const DELETE_COMMAND: &str = ":delete ";

impl DeleteCommand {
    pub(crate) fn cursor_pos(&self) -> Option<u16> {
        match self {
            Self::Init => Some(DELETE_COMMAND.len() as u16),
            Self::Typing { input } => Some(DELETE_COMMAND.len() as u16 + input.len() as u16),
            Self::Confirmed { path } => {
                Some(DELETE_COMMAND.len() as u16 + path.to_string_lossy().len() as u16)
            }
        }
    }
}
