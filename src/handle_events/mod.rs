use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};

use crate::{state::State, Result};

mod entry;

enum StateChange {
    NoActionRequired,
    ReevalOpenedPath,
    Exit,
}

impl State {
    pub fn handle_events(&mut self) -> Result<bool> {
        if !event::poll(std::time::Duration::from_millis(50))? {
            return Ok(false);
        }

        let event = event::read()?;

        if let Some(change) = self.handle_event(&event)? {
            let should_exit = self.handle_change_check_should_exit(change)?;
            return Ok(should_exit);
        }

        Ok(false)
    }

    fn handle_event(&mut self, event: &Event) -> Result<Option<StateChange>> {
        if let Some(change) = self.selected_entry_mut().handle_event(event) {
            return Ok(Some(change));
        };

        match event {
            Event::Key(key) => self.handle_key_event(key),
            _ => Ok(None),
        }
    }

    fn handle_key_event(&mut self, key: &KeyEvent) -> Result<Option<StateChange>> {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            return Ok(self
                .handle_ctrl_key_event(key.code)
                .then_some(StateChange::Exit));
        }

        if !key.modifiers.is_empty() {
            return Ok(None);
        }

        let ret = match key.code {
            KeyCode::Esc | KeyCode::Char('q') => Some(StateChange::Exit),

            KeyCode::Char('l') | KeyCode::Right => {
                self.move_right()?;
                Some(StateChange::NoActionRequired)
            }

            KeyCode::Char('h') | KeyCode::Left => {
                self.move_left()?;
                Some(StateChange::NoActionRequired)
            }

            _ => None,
        };

        Ok(ret)
    }

    fn handle_ctrl_key_event(&self, key_code: KeyCode) -> bool {
        match key_code {
            KeyCode::Char('c') => true,
            _ => false,
        }
    }

    fn handle_change_check_should_exit(&mut self, change: StateChange) -> Result<bool> {
        match change {
            StateChange::ReevalOpenedPath => self.try_open_selected_path()?,
            StateChange::Exit => return Ok(true),
            StateChange::NoActionRequired => {}
        };

        Ok(false)
    }
}
