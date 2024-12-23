use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    symbols,
    widgets::{Block, Borders, List, Paragraph, StatefulWidget, Widget},
};

use crate::ui::state::entry::opened::OpenedWidget;
use crate::Path;

mod opened;

pub struct EntryWidget {
    pub(super) selected: bool,
}

impl EntryWidget {
    fn get_opened(&self, path: Path) -> OpenedWidget {
        OpenedWidget {
            selected: self.selected,
            path,
        }
    }
}

impl StatefulWidget for EntryWidget {
    type State = crate::Entry;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut crate::Entry) {
        match &mut state.ty {
            crate::EntryType::Opened(opened) => {
                StatefulWidget::render(self.get_opened(state.path.clone()), area, buf, opened)
            }
            crate::EntryType::File => render_file(area, buf, state.path.clone()),
            crate::EntryType::Unopened(_) | crate::EntryType::Waiting(_) => {
                render_unopened(area, buf, state.path.clone())
            }
            crate::EntryType::PermissionDenied => {
                render_permission_denied(area, buf, state.path.clone())
            }
            crate::EntryType::Deleting => render_deleting(area, buf, state.path.clone()),
        }
    }
}

fn render_file(area: Rect, buf: &mut Buffer, path: Path) {
    let para = Paragraph::new(format!("file: {}", path.to_string_lossy())).block(Block::bordered());
    Widget::render(para, area, buf)
}

fn render_unopened(area: Rect, buf: &mut Buffer, path: Path) {
    let border = Block::default().borders(Borders::ALL);
    let inner = border.inner(area);
    Widget::render(border, area, buf);

    let rects = Layout::vertical([
        Constraint::Min(1),
        Constraint::Length(1),
        Constraint::Min(1),
    ])
    .split(inner);

    let para = Paragraph::new(format!("loading: {}", path.to_string_lossy())).centered();
    Widget::render(para, rects[1], buf)
}

fn render_empty_dir(area: Rect, buf: &mut Buffer, path: Path) {
    let para =
        Paragraph::new(format!("empty dir: {}", path.to_string_lossy())).block(Block::bordered());
    Widget::render(para, area, buf)
}

fn render_permission_denied(area: Rect, buf: &mut Buffer, path: Path) {
    let para = Paragraph::new(format!("🔒 permission denied: {}", path.to_string_lossy()))
        .block(Block::bordered());
    Widget::render(para, area, buf)
}

fn render_deleting(area: Rect, buf: &mut Buffer, path: Path) {
    let para =
        Paragraph::new(format!("deleting: {}", path.to_string_lossy())).block(Block::bordered());
    Widget::render(para, area, buf)
}

fn bordered_block<'a>(selected: bool) -> Block<'a> {
    let mut block = Block::bordered();

    if selected {
        block = block
            .border_style(Style::new().yellow())
            .border_set(symbols::border::THICK);
    }

    block
}

fn bordered_list<'a>(selected: bool) -> List<'a> {
    let mut highlist_style = Style::new().black().on_blue();

    if selected {
        highlist_style = highlist_style.on_yellow();
    }

    let block = bordered_block(selected);
    List::default().highlight_style(highlist_style).block(block)
}
