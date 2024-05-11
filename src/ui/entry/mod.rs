use std::{path::PathBuf, rc::Rc};

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Style, Stylize},
    symbols,
    widgets::{Block, List, Paragraph, StatefulWidget, Widget},
};

use crate::{
    state::{entry::EntryType, Entry},
    ui::entry::opened::OpenedWidget,
};

mod opened;

pub struct EntryWidget {
    pub(super) selected: bool,
}

impl EntryWidget {
    fn into_opened(&mut self, path: Rc<PathBuf>) -> OpenedWidget {
        OpenedWidget {
            selected: self.selected,
            path,
        }
    }
}

impl StatefulWidget for EntryWidget {
    type State = Entry;

    fn render(mut self, area: Rect, buf: &mut Buffer, state: &mut Entry) {
        match &mut state.ty {
            EntryType::Opened(opened) => {
                StatefulWidget::render(self.into_opened(state.path.clone()), area, buf, opened)
            }
            EntryType::File => render_file(area, buf, state.path.clone()),
            EntryType::Unopened => render_unopened(area, buf, state.path.clone()),
        }
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
