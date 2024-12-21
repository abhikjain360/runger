use std::time::Instant;

use crate::state::{Command, DeleteCommand};

#[cfg_attr(debug_assertions, derive(Debug))]
pub(crate) enum CommandPalette {
    Empty,
    Error {
        error: crate::Error,
        show_until: std::time::Instant,
    },
    Typing {
        input: String,
    },
    Command(Command),
}

impl CommandPalette {
    pub(crate) fn take(&mut self) -> CommandPalette {
        std::mem::replace(self, Self::Empty)
    }

    pub(crate) fn set_empty(&mut self) {
        self.take();
    }

    pub(crate) fn set_error(&mut self, error: crate::Error, duration: std::time::Duration) {
        *self = Self::Error {
            error,
            show_until: Instant::now() + duration,
        };
    }

    pub(crate) fn set_delete_command_init(&mut self) {
        *self = Self::Command(Command::Delete(DeleteCommand::Init));
    }

    pub(crate) fn set_delete_command_typing(&mut self, input: String) {
        *self = Self::Command(Command::Delete(DeleteCommand::Typing { input }));
    }

    pub(crate) fn cursor_pos(&self) -> Option<u16> {
        match self {
            Self::Empty | Self::Error { .. } => None,
            Self::Typing { input } => Some(input.len() as u16 + 1),
            Self::Command(command) => command.cursor_pos(),
        }
    }
}
