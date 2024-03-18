use crossterm::event::{Event, KeyCode, KeyEvent};

use crate::{
    handle_events::StateChange,
    state::{
        entry::{EntryType, Opened},
        Entry,
    },
};

impl Entry {
    pub(super) fn handle_event(&mut self, event: &Event) -> Option<StateChange> {
        match &mut self.ty {
            EntryType::Opened(opened) => opened.handle_event(event),
            _ => None,
        }
    }
}

impl Opened {
    fn handle_event(&mut self, event: &Event) -> Option<StateChange> {
        match event {
            Event::Key(key) => self.handle_key_event(key),
            _ => None,
        }
    }

    fn handle_key_event(&mut self, key: &KeyEvent) -> Option<StateChange> {
        if !key.modifiers.is_empty() {
            return None;
        }

        match key.code {
            KeyCode::Char('j') | KeyCode::Down => self.select_down_state_change(),
            KeyCode::Char('k') | KeyCode::Up => self.select_up_state_change(),
            _ => None,
        }
    }

    fn select_up_state_change(&mut self) -> Option<StateChange> {
        self.select_up().then_some(StateChange::ReevalOpenedPath)
    }

    fn select_down_state_change(&mut self) -> Option<StateChange> {
        self.select_down().then_some(StateChange::ReevalOpenedPath)
    }
}
