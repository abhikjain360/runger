use std::{path::PathBuf, rc::Rc};

#[derive(Debug, Clone)]
pub struct Opened {
    pub(crate) entries: OpenedEntries,
    pub(crate) selected: Option<Selected>,
}

#[derive(Debug, Clone)]
pub struct Selected {
    idx: usize,
    offset_from_top: usize,
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
            return Some(&entries);
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
            offset_from_top,
        }
    }

    pub fn idx(&self) -> usize {
        self.idx
    }
}
