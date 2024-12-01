use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

use ratatui::widgets::ListState;

use crate::config::Config;

#[cfg_attr(debug_assertions, derive(Debug))]
pub struct Opened {
    pub(crate) entries: Vec<Arc<PathBuf>>,
    // TODO: support multiple selection
    pub(crate) selected: Option<Selected>,
    pub(crate) config: Rc<Config>,
}

#[cfg_attr(debug_assertions, derive(Debug))]
pub struct Selected {
    idx: usize,
    offset: usize,
}

impl Opened {
    pub(crate) fn selected_entry(&self) -> Option<&Arc<PathBuf>> {
        let selected = self.selected.as_ref()?;
        Some(&self.entries[selected.idx])
    }

    pub(crate) fn select_up(&mut self) -> bool {
        let Some(mut selected) = self.selected.take() else {
            return false;
        };

        if self.entries.is_empty() {
            return false;
        }

        if selected.idx > 0 {
            selected.idx -= 1;
        } else {
            selected.idx = self.entries.len() - 1;
        }

        self.selected = Some(selected);

        true
    }

    pub(crate) fn select_down(&mut self) -> bool {
        let Some(mut selected) = self.selected.take() else {
            return false;
        };

        if self.entries.is_empty() {
            return false;
        }

        if selected.idx < self.entries.len() - 1 {
            selected.idx += 1;
        } else {
            selected.idx = 0;
        }

        self.selected = Some(selected);

        true
    }

    pub(crate) fn generate_list_state(&mut self, col_height: usize) -> ListState {
        let mut liststate = ListState::default();
        let entries_len = self.entries.len();

        if let Some(selected) = self.selected.as_mut() {
            let top_margin = self.config.column_margin.min(selected.idx);

            // if offset + margin is after idx ..
            if selected.offset + top_margin > selected.idx {
                // .. then set it to margin above idx, as offset should not be after idx ..
                selected.offset = selected.idx - top_margin;
            }
            // .. else if gap b/w offset and idx and greater than (col height - margin) then offset
            // should move down until it is is just equal to that ..
            else if selected.idx + self.config.column_margin >= selected.offset + col_height {
                selected.offset = selected.idx + self.config.column_margin - col_height + 1;

                // .. unless we have already reached the end of entries (idx is on entries_len
                // - 1), then we set it to entries_len - max_col_height - 1;
                //
                // PANIC SAFETY: substraction here is fine because if the gap b/w idx and
                // offset is greater than col height then there must be more entries in the col
                // than col height;
                selected.offset = selected.offset.min(entries_len - col_height);
            }

            liststate = liststate
                .with_selected(Some(selected.idx))
                .with_offset(selected.offset);
        }

        liststate
    }
}

impl Selected {
    pub fn new(idx: usize, offset_from_top: usize) -> Self {
        Self {
            idx,
            offset: offset_from_top,
        }
    }

    #[expect(dead_code)]
    pub fn idx(&self) -> usize {
        self.idx
    }
}
