use std::{fs, io, path::PathBuf, rc::Rc};

use crate::{
    state::entry::opened::{OpenedEntries, Selected},
    Result,
};

pub mod opened;
pub use opened::Opened;

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
                selected: entries.first().map(|_| Selected::new(0, 0)),
                entries: OpenedEntries::Entries(entries),
            }),
        }
    }
}
