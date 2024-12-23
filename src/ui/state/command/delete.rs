use ratatui::{prelude::*, widgets::Paragraph};

pub(crate) struct DeleteCommandWidget;

impl StatefulWidget for DeleteCommandWidget {
    type State = crate::DeleteCommand;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        match state {
            crate::DeleteCommand::Init => Paragraph::new(":delete ").render(area, buf),
            crate::DeleteCommand::Typing(typing) => {
                Paragraph::new(format!(":delete {}", typing.visible_query())).render(area, buf)
            }
            crate::DeleteCommand::Confirmed { path } => {
                Paragraph::new(format!(":delete {}", path.display())).render(area, buf)
            }
        }
    }
}
