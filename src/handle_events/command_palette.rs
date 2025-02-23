use crossterm::event::{KeyCode, KeyEvent};

use crate::handle_events::HandledEvent;
use crate::state::{Command, CommandPalette, DeleteCommand};
use crate::State;

impl State {
    pub(super) fn handle_command_palette_key_event(
        &mut self,
        key: &KeyEvent,
    ) -> crate::Result<HandledEvent> {
        if self.command_palette.is_empty() {
            return Ok(HandledEvent::Nothing);
        }

        match key.code {
            KeyCode::Esc => self.command_palette.set_empty(),

            KeyCode::Enter => self
                .execute_command()
                .inspect_err(|e| tracing::error!("unable to execute command: {e}"))?,

            KeyCode::Tab => self.complete_command(true),
            KeyCode::BackTab => self.complete_command(false),

            _ => return Ok(self.command_palette.handle_key_event(key)),
        };

        Ok(HandledEvent::Redraw)
    }
}

impl CommandPalette {
    pub(super) fn handle_key_event(&mut self, key: &KeyEvent) -> HandledEvent {
        match self {
            CommandPalette::Typing(typing)
            | CommandPalette::Command(Command::Delete(DeleteCommand::Typing(typing))) => {
                match key.code {
                    KeyCode::Char(c) => typing.push_char(c),
                    KeyCode::Backspace => typing.pop_char(),

                    _ => {}
                };

                HandledEvent::Redraw
            }

            CommandPalette::Command(Command::Delete(DeleteCommand::Init)) => {
                match key.code {
                    KeyCode::Char(c) => self.set_delete_command_typing(c.to_string()),

                    _ => return HandledEvent::Nothing,
                };

                HandledEvent::Redraw
            }

            _ => HandledEvent::Nothing,
        }
    }
}
