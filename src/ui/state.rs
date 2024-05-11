use std::iter;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::StatefulWidget,
};

use crate::{state::State, ui::entry::EntryWidget};

pub(crate) struct StateWidget;

impl StatefulWidget for StateWidget {
    type State = State;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut State) {
        let selected_column = state.selected_column;
        let visible_columns = state.visible_columns_mut().collect::<Vec<_>>();
        let layout = Layout::horizontal(Constraint::from_fills(
            iter::repeat(1).take(visible_columns.len()),
        ))
        .split(area);

        for (idx, (entry, area)) in visible_columns.into_iter().zip(layout.iter()).enumerate() {
            let entry_state = EntryWidget {
                selected: idx == selected_column,
            };
            entry_state.render(*area, buf, entry);
        }
    }
}
