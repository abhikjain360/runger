use std::{path::PathBuf, rc::Rc};

use ratatui::widgets::ListState;

use crate::config::Config;

#[derive(Debug, Clone)]
pub struct Opened {
    pub(crate) entries: OpenedEntries,
    pub(crate) selected: Option<Selected>,
    pub(crate) config: Rc<Config>,
}

#[derive(Debug, Clone)]
pub struct Selected {
    idx: usize,
    offset: usize,
}

#[derive(Debug, Clone)]
pub enum OpenedEntries {
    PermissionDenied,
    // TODO: add more metadata to entries, eg: dir vs file vs symlink vs executable
    Entries(Vec<Rc<PathBuf>>),
}

impl Opened {
    pub(crate) fn entries(&self) -> Option<&[Rc<PathBuf>]> {
        if let OpenedEntries::Entries(entries) = &self.entries {
            return Some(entries);
        }
        None
    }

    pub(crate) fn selected_entry(&self) -> Option<&Rc<PathBuf>> {
        let selected = self.selected.as_ref()?;
        self.entries().map(|entries| &entries[selected.idx])
    }

    /// Returns true if the path is found and set as `selected`.
    pub(crate) fn set_selected_entry(&mut self, path: &Rc<PathBuf>) -> bool {
        let Some(idx) = self.entries().and_then(|entries| {
            entries
                .iter()
                .enumerate()
                .find_map(|(idx, entry)| (entry == path).then_some(idx))
        }) else {
            return false;
        };

        let mut selected = self.selected.take().unwrap_or_else(|| Selected::new(0, 0));
        selected.idx = idx;
        self.selected = Some(selected);

        false
    }

    pub(crate) fn select_up(&mut self) -> bool {
        let (Some(mut selected), Some(entries)) = (self.selected.take(), self.entries()) else {
            return false;
        };

        if entries.is_empty() {
            return false;
        }

        if selected.idx > 0 {
            selected.idx -= 1;
        } else {
            selected.idx = entries.len() - 1;
        }

        self.selected = Some(selected);

        true
    }

    pub(crate) fn select_down(&mut self) -> bool {
        let (Some(mut selected), Some(entries)) = (self.selected.take(), self.entries()) else {
            return false;
        };

        if entries.is_empty() {
            return false;
        }

        if selected.idx < entries.len() - 1 {
            selected.idx += 1;
        } else {
            selected.idx = 0;
        }

        self.selected = Some(selected);

        true
    }

    pub(crate) fn generate_list_state(&mut self, max_col_height: usize) -> ListState {
        let mut liststate = ListState::default();
        let entries_len = self.entries().map(|entries| entries.len()).unwrap_or(0);

        if let Some(selected) = self.selected.as_mut() {
            let top_margin = self.config.column_margin.min(selected.idx);

            // if offset + margin is after idx ..
            if selected.offset + top_margin > selected.idx {
                // .. then set it to margin above idx, as offset should not be after idx ..
                selected.offset = selected.idx - top_margin;
            }
            // .. else if gap b/w offset and idx and greater than (col height - margin) then offset
            // should move down until it is is just equal to that ..
            else {
                if selected.idx + self.config.column_margin >= selected.offset + max_col_height {
                    selected.offset = selected.idx + self.config.column_margin - max_col_height + 1;

                    // .. unless we have already reached the end of entries (idx is on entries_len
                    // - 1), then we set it to entries_len - max_col_height - 1;
                    selected.offset = selected.offset.min(entries_len - max_col_height);
                }
            }

            liststate = liststate
                .with_selected(Some(selected.idx))
                .with_offset(selected.offset);
        }

        liststate
    }
}

impl OpenedEntries {
    pub fn is_empty(&self) -> bool {
        match self {
            OpenedEntries::PermissionDenied => false,
            OpenedEntries::Entries(entries) => entries.is_empty(),
        }
    }
}

impl Selected {
    pub fn new(idx: usize, offset_from_top: usize) -> Self {
        Self {
            idx,
            offset: offset_from_top,
        }
    }

    pub fn idx(&self) -> usize {
        self.idx
    }
}
