use std::iter;

use ratatui::prelude::*;

use crate::ui::state::command_palette::CommandPaletteWidget;
use crate::ui::state::entry::EntryWidget;

mod command_palette;
mod entry;

pub(crate) struct StateWidget;

impl StatefulWidget for StateWidget {
    type State = crate::State;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let columns_area = match state.command_palette {
            crate::state::CommandPalette::Empty => area,
            _ => {
                let layout =
                    Layout::vertical([Constraint::Min(3), Constraint::Length(1)]).split(area);

                CommandPaletteWidget.render(layout[1], buf, &mut state.command_palette);

                layout[0]
            }
        };

        let selected_column = state.selected_column;
        let visible_columns = state.visible_columns_mut().collect::<Vec<_>>();

        let columns_layout = Layout::horizontal(Constraint::from_fills(
            iter::repeat(1).take(visible_columns.len()),
        ))
        .split(columns_area);

        for (idx, (entry, area)) in visible_columns
            .into_iter()
            .zip(columns_layout.iter())
            .enumerate()
        {
            let entry_state = EntryWidget {
                selected: idx == selected_column,
            };
            entry_state.render(*area, buf, entry);
        }
    }
}
