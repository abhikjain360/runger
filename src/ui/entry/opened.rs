use std::{path::PathBuf, rc::Rc};

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    text::Text,
    widgets::{ListState, Paragraph, StatefulWidget, Widget},
};

use crate::{
    state::entry::{
        opened::{OpenedEntries, Selected},
        Opened,
    },
    ui::entry::{bordered_list, render_unopened},
};

pub struct OpenedWidget {
    pub(super) selected: bool,
    pub(super) path: Rc<PathBuf>,
}

impl StatefulWidget for OpenedWidget {
    type State = Opened;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Opened) {
        let entries = match &state.entries {
            OpenedEntries::Entries(entries) => entries,
            OpenedEntries::PermissionDenied => {
                let paragraph = Paragraph::new("🔒 Permission Denied");
                Widget::render(paragraph, area, buf);
                return;
            }
        };

        let selected_idx_opt = state.selected.as_ref().map(Selected::idx);
        let mut list_state = ListState::default().with_selected(selected_idx_opt);

        if !entries.is_empty() {
            let list = bordered_list(self.selected).items(entries.iter().filter_map(|entry| {
                let file_name = entry.file_name()?.to_string_lossy().into_owned();

                let mut text = Text::from(file_name);
                if entry.is_dir() {
                    text = text.blue().bold();
                }

                Some(text)
            }));
            StatefulWidget::render(list, area, buf, &mut list_state);
            return;
        }

        render_unopened(area, buf, self.path.clone())
    }
}
