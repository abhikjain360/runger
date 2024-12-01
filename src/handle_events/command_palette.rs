use crossterm::event::{KeyCode, KeyEvent};

use crate::handle_events::StateChange;
use crate::state::CommandPalette;

impl CommandPalette {
    pub(super) fn handle_key_event(&mut self, key: &KeyEvent) -> Option<StateChange> {
        match self {
            CommandPalette::Typing { input } => {
                match key.code {
                    KeyCode::Char(c) => {
                        input.push(c);
                    }
                    KeyCode::Backspace => {
                        input.pop();
                    }
                    KeyCode::Esc => {
                        *self = CommandPalette::Empty;
                    }
                    _ => {}
                };
                Some(StateChange::NoActionRequired)
            }

            _ => None,
        }
    }
}
