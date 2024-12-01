pub(crate) use crate::state::command::delete::DeleteCommand;

use crate::state::{CommandPalette, State};

mod delete;

#[cfg_attr(debug_assertions, derive(Debug))]
pub(crate) enum Command {
    Delete(DeleteCommand),
}

#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(thiserror::Error)]
pub enum CommandError {
    #[error("Invalid command")]
    InvalidCommand,
}

impl State {
    #[tracing::instrument(err, level = "trace", skip(self))]
    pub(crate) fn execute_command(&mut self) -> crate::Result<()> {
        match self.command_palette.take() {
            CommandPalette::Command(Command::Delete(DeleteCommand::Confirmed { path })) => {
                self.delete_path(&path)
            }

            CommandPalette::Command(Command::Delete(DeleteCommand::Init)) => {
                let entry = self.selected_entry();
                let path = match &entry.ty {
                    crate::EntryType::Opened(opened) => match opened.selected_entry() {
                        Some(selected_entry) => selected_entry.clone(),
                        None => {
                            tracing::warn!(
                                "attempted to delete without selecting an entry in the column"
                            );
                            return Err(CommandError::InvalidCommand.into());
                        }
                    },

                    crate::EntryType::File => entry.path.clone(),

                    _ => return Ok(()),
                };
                self.delete_path(&path);
            }

            // TODO: support parsing input as command
            CommandPalette::Typing { input: _ } => todo!(),

            _ => {}
        }

        Ok(())
    }
}
