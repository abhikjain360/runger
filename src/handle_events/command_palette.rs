use crossterm::event::{KeyCode, KeyEvent};

use crate::handle_events::StateChange;
use crate::state::{Command, CommandPalette, DeleteCommand};

impl CommandPalette {
    pub(super) fn handle_key_event(&mut self, key: &KeyEvent) -> Option<StateChange> {
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

                    KeyCode::Tab => {
                        return Some(StateChange::TryCommandCompletion);
                    }

                    KeyCode::Enter => {
                        return Some(StateChange::ExecuteCommand);
                    }

                    _ => {}
                };
                Some(StateChange::NoActionRequired)
            }

            CommandPalette::Command(Command::Delete(DeleteCommand::Init)) => match key.code {
                KeyCode::Char(c) => {
                    self.set_delete_command_typing(c.to_string());
                    Some(StateChange::NoActionRequired)
                }

                KeyCode::Esc => {
                    self.set_empty();
                    Some(StateChange::NoActionRequired)
                }

                KeyCode::Enter => Some(StateChange::ExecuteCommand),
                _ => Some(StateChange::NoActionRequired),
            },

            _ => None,
        }
    }
}
