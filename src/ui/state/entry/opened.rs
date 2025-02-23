use ratatui::{buffer::Buffer, layout::Rect, style::Stylize, text::Text, widgets::StatefulWidget};

use crate::{
    path::Path,
    state::entry::Opened,
    ui::state::entry::{bordered_list, render_empty_dir},
};

pub struct OpenedWidget {
    pub(super) selected: bool,
    pub(super) path: Path,
}

impl StatefulWidget for OpenedWidget {
    type State = Opened;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Opened) {
        let mut list_state = state.generate_list_state(2.max(area.height as usize) - 2);

        if state.entries.is_empty() {
            render_empty_dir(area, buf, self.path.clone());
            return;
        }

        let list =
            bordered_list(self.selected).items(state.entries.iter().filter_map(path_formatting));
        StatefulWidget::render(list, area, buf, &mut list_state);
    }
}

fn path_formatting(path: &Path) -> Option<Text> {
    let file_name = path.file_name()?.to_string_lossy().into_owned();

    let mut text = Text::from(file_name);
    text = if path.exists() {
        match (path.is_dir(), path.is_symlink()) {
            (true, true) => text.light_green().bold(),
            (true, false) => text.blue().bold(),
            (false, true) => text.light_green(),
            (false, false) => text,
        }
    } else {
        // path is probably deleting
        text.gray()
    };

    Some(text)
}
