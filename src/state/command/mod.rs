pub(crate) use crate::state::command::delete::DeleteCommand;

use crate::state::{CommandPalette, State};
use crate::Path;

mod completion;
mod delete;

#[cfg_attr(debug_assertions, derive(Debug))]
pub(crate) enum Command {
    Delete(DeleteCommand),
}

#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("Invalid command")]
    InvalidCommand,
    #[error("Invalid path")]
    InvalidPath,
}

impl State {
    #[tracing::instrument(err, level = "trace", skip(self))]
    pub(crate) fn execute_command(&mut self) -> Result<(), CommandError> {
        #[expect(clippy::single_match)]
        match self.command_palette.take() {
            CommandPalette::Command(Command::Delete(delete_command)) => {
                let path = match delete_command {
                    DeleteCommand::Confirmed { path } => path,

                    DeleteCommand::Init => {
                        let Some(opened) = self.selected_entry().get_opened() else {
                            tracing::error!("attempted to delete from unopened entry");
                            return Err(CommandError::InvalidCommand);
                        };

                        let Some(selected_entry) = opened.selected_entry() else {
                            tracing::error!(
                                "attempted to delete without selecting an entry in the column"
                            );
                            return Err(CommandError::InvalidCommand);
                        };

                        selected_entry.clone()
                    }

                    DeleteCommand::Typing(typing) => self
                        .match_file_path(typing.visible_query())
                        .ok_or(CommandError::InvalidPath)?,
                };

                self.delete_path(path)
            }

            _ => {}
        };

        Ok(())
    }

    fn match_file_path(&self, path: impl AsRef<std::path::Path>) -> Option<Path> {
        let input_path = path.as_ref();
        let opened = self.selected_entry().get_opened()?;

        opened
            .entries
            .iter()
            .find(|entry| {
                entry
                    .as_path()
                    .file_name()
                    .and_then(|file_name| file_name.to_str())
                    .map_or(false, |file_name| file_name == input_path.to_string_lossy())
            })
            .cloned()
    }
}

impl Command {
    pub(crate) fn cursor_pos(&self) -> u16 {
        match self {
            Self::Delete(delete) => delete.cursor_pos(),
        }
    }
}
