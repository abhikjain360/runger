use std::{fs, io, path::PathBuf, rc::Rc};

use crate::Result;

#[derive(Debug, Clone)]
pub enum EntryType {
    Opened(Opened),
    File,
    Unopened,
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub path: Rc<PathBuf>,
    pub ty: EntryType,
}

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

impl Entry {
    pub(crate) fn new(path: Rc<PathBuf>) -> Self {
        let mut ret = Self {
            path: path.clone(),
            ty: EntryType::File,
        };
        if path.is_dir() {
            ret.ty = EntryType::Unopened;
        }
        ret
    }

    pub(crate) fn try_open(&mut self) -> Result<Option<&Opened>> {
        let path = match self.ty {
            EntryType::File => return Ok(None),
            EntryType::Opened(ref mut opened) => return Ok(Some(opened)),
            EntryType::Unopened => &self.path,
        };

        let entries = match fs::read_dir(path.as_ref()) {
            Err(e) if e.kind() == io::ErrorKind::PermissionDenied => return Ok(None),
            res => res?,
        };

        let mut entries = entries
            .map(|entry_res| Ok(Rc::new(entry_res?.path().to_path_buf())))
            .collect::<Result<Vec<_>>>()?;
        entries.sort();

        *self = Self::opened(path.clone(), entries);

        self.try_open()
    }

    pub(crate) fn opened(path: Rc<PathBuf>, entries: Vec<Rc<PathBuf>>) -> Self {
        Self {
            path,
            ty: EntryType::Opened(Opened {
                selected: entries.first().map(|_| 0),
                entries: OpenedEntries::Entries(entries),
            }),
        }
    }
}

impl EntryType {
    pub fn is_file(&self) -> bool {
        matches!(self, EntryType::File)
    }
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
