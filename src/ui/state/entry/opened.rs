use std::path::PathBuf;
use std::sync::Arc;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    text::Text,
    widgets::{Paragraph, StatefulWidget, Widget},
};

use crate::{
    state::entry::{opened::OpenedEntries, Opened},
    ui::state::entry::{bordered_list, render_empty_dir},
};

pub struct OpenedWidget {
    pub(super) selected: bool,
    pub(super) path: Arc<PathBuf>,
}

impl StatefulWidget for OpenedWidget {
    type State = Opened;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Opened) {
        let mut list_state = state.generate_list_state(2.max(area.height as usize) - 2);

        let entries = match &state.entries {
            OpenedEntries::Entries(entries) if !entries.is_empty() => entries,
            OpenedEntries::Entries(_) => {
                render_empty_dir(area, buf, self.path.clone());
                return;
            }
            OpenedEntries::PermissionDenied => {
                let paragraph = Paragraph::new("🔒 Permission Denied");
                Widget::render(paragraph, area, buf);
                return;
            }
        };

        let list = bordered_list(self.selected).items(entries.iter().filter_map(path_formatting));
        StatefulWidget::render(list, area, buf, &mut list_state);
    }
}

fn path_formatting(path: &Arc<PathBuf>) -> Option<Text> {
    let file_name = path.file_name()?.to_string_lossy().into_owned();

    let mut text = Text::from(file_name);
    text = match (path.is_dir(), path.is_symlink()) {
        (true, true) => text.light_green().bold(),
        (true, false) => text.blue().bold(),
        (false, true) => text.light_green(),
        (false, false) => text,
    };

    Some(text)
}
