use std::{path::PathBuf, rc::Rc};

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Style, Stylize},
    symbols,
    text::Text,
    widgets::{Block, List, ListState, Paragraph, StatefulWidget, StatefulWidgetRef, Widget},
};

use crate::state::{
    entry::{opened::OpenedEntries, EntryType, Opened},
    Entry,
};

pub struct EntryState {
    pub(super) selected: bool,
}

pub struct OpenedState {
    selected: bool,
    path: Rc<PathBuf>,
}

impl EntryState {
    fn into_opened(&mut self, path: Rc<PathBuf>) -> OpenedState {
        OpenedState {
            selected: self.selected,
            path,
        }
    }
}

impl StatefulWidgetRef for &Entry {
    type State = EntryState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut EntryState) {
        match &self.ty {
            EntryType::Opened(opened) => StatefulWidgetRef::render_ref(
                opened,
                area,
                buf,
                &mut state.into_opened(self.path.clone()),
            ),
            EntryType::File => render_file(area, buf, self.path.clone()),
            EntryType::Unopened => render_unopened(area, buf, self.path.clone()),
        }
    }
}

impl StatefulWidgetRef for Opened {
    type State = OpenedState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut OpenedState) {
        let entries = match &self.entries {
            OpenedEntries::Entries(entries) => entries,
            OpenedEntries::PermissionDenied => {
                let paragraph = Paragraph::new("🔒 Permission Denied");
                Widget::render(paragraph, area, buf);
                return;
            }
        };

        let mut list_state = ListState::default().with_selected(self.selected);

        if !entries.is_empty() {
            let list = bordered_list(state.selected).items(entries.iter().filter_map(|entry| {
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

        render_unopened(area, buf, state.path.clone())
    }
}

fn render_file(area: Rect, buf: &mut Buffer, path: Rc<PathBuf>) {
    let para = Paragraph::new(format!("file: {}", path.to_string_lossy())).block(Block::bordered());
    Widget::render(para, area, buf)
}

fn render_unopened(area: Rect, buf: &mut Buffer, path: Rc<PathBuf>) {
    let para =
        Paragraph::new(format!("empty dir: {}", path.to_string_lossy())).block(Block::bordered());
    Widget::render(para, area, buf)
}

fn bordered_list<'a>(selected: bool) -> List<'a> {
    let mut block = Block::bordered();
    let mut highlist_style = Style::new().black().on_blue();

    if selected {
        block = block
            .border_style(Style::new().yellow())
            .border_set(symbols::border::THICK);

        highlist_style = highlist_style.on_yellow();
    }

    List::default().highlight_style(highlist_style).block(block)
}
