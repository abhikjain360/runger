use crate::state::command_palette::Typing;
use crate::Path;

// TODO: support deleting multiple entries
#[cfg_attr(debug_assertions, derive(Debug))]
pub(crate) enum DeleteCommand {
    Init,
    Typing(Typing),
    #[expect(dead_code)]
    Confirmed {
        path: Path,
    },
}

pub const DELETE_COMMAND: &str = ":delete ";

impl DeleteCommand {
    pub(crate) fn cursor_pos(&self) -> u16 {
        (match self {
            Self::Init => 0_u16,
            Self::Typing(typing) => typing.cursor_pos(),
            Self::Confirmed { path } => path.to_string_lossy().len() as u16,
        }) + DELETE_COMMAND.len() as u16
    }
}
