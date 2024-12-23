use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};

use crate::{
    state::{CommandPalette, State},
    Result,
};

mod command_palette;
mod entry;
mod joiners;

const MAX_EVENT_POLL_TIME: Duration = Duration::from_millis(1000 / 120);

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum HandledEvent {
    Exit,
    Redraw,
    Nothing,
}

impl State {
    pub fn handle_events(&mut self) -> Result<HandledEvent> {
        let start = Instant::now();
        let mut elapsed = start.elapsed();

        let mut ret = HandledEvent::Nothing;

        // poll events until MAX_EVENT_POLL_TIME is reached
        while elapsed < MAX_EVENT_POLL_TIME {
            if event::poll(Duration::from_millis(1))? {
                let event = event::read()?;

                // handle TUI events first for smoother UX
                match self.handle_tui_event(&event) {
                    HandledEvent::Exit => return Ok(HandledEvent::Exit),
                    // not returning here as we still want to poll IO events and drive the async
                    // runtime
                    HandledEvent::Redraw => ret = HandledEvent::Redraw,

                    HandledEvent::Nothing => {}
                };
            }

            elapsed = start.elapsed();

            // return if no time left for IO events
            if elapsed >= MAX_EVENT_POLL_TIME {
                return Ok(ret);
            }

            let ret = self.poll_io_event(MAX_EVENT_POLL_TIME / 2)?;
            if ret.is_handled() {
                return Ok(ret);
            }
            elapsed = start.elapsed();
        }

        Ok(ret)
    }

    fn handle_tui_event(&mut self, event: &Event) -> HandledEvent {
        if let Event::Key(key) = event {
            let ret = self.handle_key_event(key);
            if ret.is_handled() {
                return ret;
            }
        }

        if self.selected_entry_mut().handle_event(event) {
            self.try_open_selected_path();
            return HandledEvent::Redraw;
        }

        HandledEvent::Nothing
    }

    fn handle_key_event(&mut self, key: &KeyEvent) -> HandledEvent {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            let ret = self.handle_ctrl_key_event(key.code);
            if ret.is_handled() {
                return ret;
            }
        }

        match self.handle_command_palette_key_event(key) {
            Ok(ret) if ret.is_handled() => return ret,
            Ok(_) => {}
            Err(e) => {
                self.command_palette.set_error(e, Duration::from_secs(5));
                return HandledEvent::Redraw;
            }
        };

        if !key.modifiers.is_empty() {
            return HandledEvent::Nothing;
        }

        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => return HandledEvent::Exit,

            KeyCode::Char('l') | KeyCode::Right => _ = self.move_right(),

            KeyCode::Char('h') | KeyCode::Left => _ = self.move_left(),

            KeyCode::Char(';') => {
                self.command_palette = CommandPalette::Typing {
                    input: String::new(),
                }
            }

            KeyCode::Char('d') => self.command_palette.set_delete_command_init(),

            _ => return HandledEvent::Nothing,
        };

        HandledEvent::Redraw
    }

    fn handle_ctrl_key_event(&self, key_code: KeyCode) -> HandledEvent {
        match key_code {
            KeyCode::Char('c') => HandledEvent::Exit,
            _ => HandledEvent::Nothing,
        }
    }
}

impl HandledEvent {
    pub fn is_handled(&self) -> bool {
        !matches!(self, HandledEvent::Nothing)
    }
}
