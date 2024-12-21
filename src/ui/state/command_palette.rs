use std::time::Instant;

use ratatui::{prelude::*, widgets::Paragraph};

use crate::state::CommandPalette;
use crate::ui::state::command::CommandWidget;

pub(crate) struct CommandPaletteWidget;

impl StatefulWidget for CommandPaletteWidget {
    type State = CommandPalette;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        match state {
            CommandPalette::Empty => {}
            CommandPalette::Error { error, show_until } => {
                let now = Instant::now();
                if now >= *show_until {
                    *state = CommandPalette::Empty;
                    return;
                }

                // TODO: error in red
                let error = format!("error: {}", error);
                Paragraph::new(error).render(area, buf);
            }
            CommandPalette::Typing { input } => {
                let input = format!(":{}", input);
                Paragraph::new(input).render(area, buf);
            }
            CommandPalette::Command(command) => CommandWidget.render(area, buf, command),
        }
    }
}
