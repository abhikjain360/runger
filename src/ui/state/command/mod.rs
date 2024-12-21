use ratatui::prelude::*;

use crate::ui::state::command::delete::DeleteCommandWidget;

mod delete;

pub(crate) struct CommandWidget;

impl StatefulWidget for CommandWidget {
    type State = crate::Command;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        match state {
            crate::Command::Delete(delete_command) => {
                DeleteCommandWidget.render(area, buf, delete_command)
            }
        }
    }
}
