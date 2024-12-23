use crate::state::command_palette::Typing;
use crate::state::{CommandPalette, DeleteCommand};
use crate::{Command, Path, State};

impl State {
    pub(crate) fn complete_command(&mut self, next: bool) {
        // SAFETY: we do not borrow self.command_palette again
        let command_palette = unsafe {
            std::mem::transmute::<&mut CommandPalette, &mut CommandPalette>(
                &mut self.command_palette,
            )
        };

        match command_palette {
            CommandPalette::Command(Command::Delete(DeleteCommand::Typing(typing))) => {
                if typing.has_completion() {
                    typing.select_completion(next);
                    return;
                }

                if let Some(opened) = self.selected_entry().get_opened() {
                    filter_completions(&opened.entries, typing, next);
                }
            }

            CommandPalette::Command(Command::Delete(DeleteCommand::Init)) => {
                if let Some(opened) = self.selected_entry().get_opened() {
                    let mut typing = Typing::default();
                    filter_completions(&opened.entries, &mut typing, next);

                    self.command_palette =
                        CommandPalette::Command(Command::Delete(DeleteCommand::Typing(typing)));
                }
            }

            _ => {}
        }
    }
}

fn filter_completions(entries: &[Path], typing: &mut Typing, next: bool) {
    let completions = entries
        .iter()
        .filter_map(|entry| {
            let file_name = entry.file_name()?.to_string_lossy();
            file_name
                .starts_with(typing.visible_query())
                .then_some(file_name.to_string())
        })
        .collect::<Vec<_>>();

    typing.set_completion_candidates(completions);
    typing.select_completion(next);
}
