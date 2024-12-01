use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};

use crate::{
    state::{CommandPalette, State},
    Result,
};

mod command_palette;
mod entry;
mod joiners;

enum StateChange {
    NoActionRequired,
    ReEvalOpenedPath,
    // TODO: command completion rotations
    TryCommandCompletion,
    ExecuteCommand,
    Exit,
}

const MAX_EVENT_POLL_TIME: Duration = Duration::from_millis(1000 / 120);

impl State {
    pub fn handle_events(&mut self) -> Result<bool> {
        let start = Instant::now();
        let mut elapsed = start.elapsed();

        // poll events until MAX_EVENT_POLL_TIME is reached
        while elapsed < MAX_EVENT_POLL_TIME {
            if event::poll(MAX_EVENT_POLL_TIME / 2)? {
                let event = event::read()?;

                // handle TUI events first for smoother UX
                if let Some(change) = self.handle_tui_event(&event) {
                    let should_exit = self.handle_change_check_should_exit(change)?;
                    return Ok(should_exit);
                }
            }

            elapsed = start.elapsed();

            // return if no time left for IO events
            if elapsed >= MAX_EVENT_POLL_TIME {
                return Ok(false);
            }

            if let Some(change) = self.poll_io_event(MAX_EVENT_POLL_TIME / 2)? {
                let should_exit = self.handle_change_check_should_exit(change)?;
                return Ok(should_exit);
            }
            elapsed = start.elapsed();
        }

        Ok(false)
    }

    fn handle_tui_event(&mut self, event: &Event) -> Option<StateChange> {
        let selected_entry = self.selected_entry_mut();

        if let Some(change) = selected_entry.handle_event(event) {
            return Some(change);
        };

        match event {
            Event::Key(key) => self.handle_key_event(key),
            _ => None,
        }
    }

    fn handle_key_event(&mut self, key: &KeyEvent) -> Option<StateChange> {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            return self
                .handle_ctrl_key_event(key.code)
                .then_some(StateChange::Exit);
        }

        if let Some(change) = self.command_palette.handle_key_event(key) {
            return Some(change);
        }

        if !key.modifiers.is_empty() {
            return None;
        }

        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => Some(StateChange::Exit),

            KeyCode::Char('l') | KeyCode::Right => {
                self.move_right();
                Some(StateChange::NoActionRequired)
            }

            KeyCode::Char('h') | KeyCode::Left => {
                self.move_left();
                Some(StateChange::NoActionRequired)
            }

            KeyCode::Char(';') => {
                self.command_palette = CommandPalette::Typing {
                    input: String::new(),
                };
                Some(StateChange::NoActionRequired)
            }

            KeyCode::Char('d') => {
                self.command_palette.set_delete_command_init();
                Some(StateChange::NoActionRequired)
            }

            _ => None,
        }
    }

    fn handle_ctrl_key_event(&self, key_code: KeyCode) -> bool {
        #[allow(clippy::match_like_matches_macro)]
        match key_code {
            KeyCode::Char('c') => true,
            _ => false,
        }
    }

    fn handle_change_check_should_exit(&mut self, change: StateChange) -> Result<bool> {
        match change {
            StateChange::ReEvalOpenedPath => _ = self.try_open_selected_path(),

            StateChange::TryCommandCompletion => todo!(),
            StateChange::ExecuteCommand => self.execute_command()?,

            StateChange::NoActionRequired => {}
            StateChange::Exit => return Ok(true),
        }

        Ok(false)
    }
}
