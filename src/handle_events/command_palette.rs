use crossterm::event::{KeyCode, KeyEvent};

use crate::state::{Command, CommandPalette, DeleteCommand};

impl CommandPalette {
    /// Returns `Some(true)` if the command should be executed. Returns `None` if the event was not
    /// handled.
    pub(super) fn handle_key_event(&mut self, key: &KeyEvent) -> Option<bool> {
        match self {
            CommandPalette::Typing { input }
            | CommandPalette::Command(Command::Delete(DeleteCommand::Typing { input })) => {
                match key.code {
                    KeyCode::Char(c) => {
                        input.push(c);
                    }

                    KeyCode::Backspace => {
                        input.pop();
                    }

                    KeyCode::Esc => self.set_empty(),

                    // TODO: command completion rotations
                    KeyCode::Tab => {}

                    KeyCode::Enter => {
                        return Some(true);
                    }

                    _ => {}
                };
                Some(false)
            }

            CommandPalette::Command(Command::Delete(DeleteCommand::Init)) => match key.code {
                KeyCode::Char(c) => {
                    self.set_delete_command_typing(c.to_string());
                    Some(false)
                }

                KeyCode::Esc => {
                    self.set_empty();
                    Some(false)
                }

                KeyCode::Enter => Some(true),
                _ => Some(false),
            },

            _ => None,
        }
    }
}
