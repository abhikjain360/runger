use std::{path::PathBuf, rc::Rc};

#[derive(Debug, Clone)]
pub struct Opened {
    pub(crate) entries: OpenedEntries,
    pub(crate) selected: Option<usize>,
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
        let selected = self.selected.as_ref().copied()?;
        self.entries().map(|entries| &entries[selected])
    }

    /// Returns true if the path is found and set as `selected`.
    pub(crate) fn set_selected_entry(&mut self, path: &Rc<PathBuf>) -> bool {
        let res = self.entries().and_then(|entries| {
            entries
                .iter()
                .enumerate()
                .find_map(|(idx, entry)| (entry == path).then_some(idx))
        });
        if res.is_some() {
            self.selected = res;
            return true;
        }
        false
    }

    pub(crate) fn select_up(&mut self) -> bool {
        let (Some(mut selected), Some(entries)) = (self.selected.take(), self.entries()) else {
            return false;
        };

        if entries.is_empty() {
            return false;
        }

        if selected > 0 {
            selected -= 1;
        } else {
            selected = entries.len() - 1;
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

        if selected < entries.len() - 1 {
            selected += 1;
        } else {
            selected = 0;
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
