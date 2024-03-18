use ratatui::Frame;

use crate::state::State;

mod entry;
mod state;

impl State {
    pub fn ui(&self) -> impl for<'b> FnOnce(&'b mut Frame<'_>) + '_ {
        move |frame| frame.render_widget_ref(self, frame.size())
    }
}
