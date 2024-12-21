use ratatui::Frame;

use crate::state::State;

use self::state::StateWidget;

mod state;

impl State {
    pub fn ui(&mut self) -> impl for<'b> FnOnce(&'b mut Frame<'_>) + '_ {
        move |frame| {
            frame.render_stateful_widget(StateWidget, frame.area(), self);
            if let Some(x) = self.command_palette.cursor_pos() {
                frame.set_cursor_position((x, self.command_palette_row));
            }
        }
    }
}
