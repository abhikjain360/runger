use crossterm::event::{Event, KeyCode, KeyEvent};

use crate::state::entry;

impl crate::Entry {
    pub(super) fn handle_event(&mut self, event: &Event) -> bool {
        match &mut self.ty {
            crate::EntryType::Opened(opened) => opened.handle_event(event),
            _ => false,
        }
    }
}

impl entry::Opened {
    fn handle_event(&mut self, event: &Event) -> bool {
        match event {
            Event::Key(key) => self.handle_key_event(key),
            _ => false,
        }
    }

    fn handle_key_event(&mut self, key: &KeyEvent) -> bool {
        if !key.modifiers.is_empty() {
            return false;
        }

        match key.code {
            KeyCode::Char('j') | KeyCode::Down => self.select_down(),
            KeyCode::Char('k') | KeyCode::Up => self.select_up(),
            _ => false,
        }
    }
}
