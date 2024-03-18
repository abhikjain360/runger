use std::iter;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::{StatefulWidgetRef, WidgetRef},
};

use crate::{state::State, ui::entry::EntryState};

impl WidgetRef for &State {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let visible_columns = self.visible_columns().collect::<Vec<_>>();
        let layout = Layout::horizontal(Constraint::from_fills(
            iter::repeat(1).take(visible_columns.len()),
        ))
        .split(area);

        for (idx, (entry, area)) in visible_columns
            .into_iter()
            .zip(layout.into_iter())
            .enumerate()
        {
            let entry_state = &mut EntryState {
                selected: idx == self.selected_column,
            };
            entry.render_ref(*area, buf, entry_state);
        }
    }
}
