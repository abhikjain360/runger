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
pub(crate) struct Selected {
    path: Arc<PathBuf>,
    display_offset: usize,
}

impl Opened {
    pub(crate) fn selected_entry(&self) -> Option<&Arc<PathBuf>> {
        self.selected.as_ref().map(|selected| &selected.path)
    }

    pub(crate) fn selected_entry_idx_and_offset(&self) -> Option<(usize, usize)> {
        self.selected.as_ref().and_then(|selected| {
            self.entries
                .iter()
                .position(|e| e == &selected.path)
                .map(|idx| (idx, selected.display_offset))
        })
    }

    pub(crate) fn set_selected(&mut self, idx: usize, offset: usize) {
        self.selected = Some(Selected {
            path: self.entries[idx].clone(),
            display_offset: offset,
        });
    }

    pub(crate) fn select_up(&mut self) -> bool {
        if self.entries.is_empty() {
            return false;
        }

        let Some((mut idx, offset)) = self.selected_entry_idx_and_offset() else {
            return false;
        };

        if idx > 0 {
            idx -= 1;
        } else {
            idx = self.entries.len() - 1;
        }

        self.set_selected(idx, offset);

        true
    }

    pub(crate) fn select_down(&mut self) -> bool {
        if self.entries.is_empty() {
            return false;
        }

        let Some((mut idx, offset)) = self.selected_entry_idx_and_offset() else {
            return false;
        };

        if idx < self.entries.len() - 1 {
            idx += 1;
        } else {
            idx = 0;
        }

        self.set_selected(idx, offset);

        true
    }

    pub(crate) fn generate_list_state(&mut self, col_height: usize) -> ListState {
        let mut liststate = ListState::default();
        let entries_len = self.entries.len();

        if let Some((idx, mut offset)) = self.selected_entry_idx_and_offset() {
            let top_margin = self.config.column_margin.min(idx);

            // if offset + margin is after idx ..
            if offset + top_margin > idx {
                // .. then set it to margin above idx, as offset should not be after idx ..
                offset = idx.max(top_margin) - top_margin;
            }
            // .. else if gap b/w offset and idx and greater than (col height - margin) then offset
            // should move down until it is is just equal to that ..
            else if idx + self.config.column_margin >= offset + col_height {
                offset = idx + self.config.column_margin - col_height + 1;

                // .. unless we have already reached the end of entries (idx is on entries_len
                // - 1), then we set it to entries_len - max_col_height - 1;
                //
                // PANIC SAFETY: substraction here is fine because if the gap b/w idx and
                // offset is greater than col height then there must be more entries in the col
                // than col height;
                offset = offset.min(entries_len - col_height);
            }

            self.set_selected(idx, offset);

            liststate = liststate.with_selected(Some(idx)).with_offset(offset);
        } else {
            tracing::warn!("generating list state without selected entry");
        }

        liststate
    }
}

impl Selected {
    pub fn new(path: Arc<PathBuf>, offset_from_top: usize) -> Self {
        Self {
            path,
            display_offset: offset_from_top,
        }
    }

    pub fn path(&self) -> &Arc<PathBuf> {
        &self.path
    }
}
