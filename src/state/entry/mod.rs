use std::{fs, io, path::PathBuf, rc::Rc};

use crate::{
    config::Config,
    state::entry::opened::{OpenedEntries, Selected},
    Result,
};

pub mod opened;
pub use opened::Opened;

pub enum EntryType {
    Opened(Opened),
    File,
    Unopened,
}

pub struct Entry {
    pub path: Rc<PathBuf>,
    pub ty: EntryType,
    pub config: Rc<Config>,
}

impl Entry {
    pub(crate) fn new(path: Rc<PathBuf>, config: Rc<Config>) -> Self {
        let mut ret = Self {
            path: path.clone(),
            ty: EntryType::File,
            config,
        };
        if path.is_dir() {
            ret.ty = EntryType::Unopened;
        }
        ret
    }

    pub(crate) fn try_open(&mut self) -> Result<Option<&Opened>> {
        match self.ty {
            EntryType::File => return Ok(None),
            EntryType::Opened(ref mut opened) => return Ok(Some(opened)),
            EntryType::Unopened => {}
        };

        let entries = match fs::read_dir(self.path.as_ref()) {
            Err(e) if e.kind() == io::ErrorKind::PermissionDenied => return Ok(None),
            res => res?,
        };

        let mut entries = entries
            .map(|entry_res| Ok(Rc::new(entry_res?.path().to_path_buf())))
            .collect::<Result<Vec<_>>>()?;
        entries.sort();

        self.ty = EntryType::Opened(Opened {
            selected: entries.first().map(|_| Selected::new(0, 0)),
            entries: OpenedEntries::Entries(entries),
            config: self.config.clone(),
        });

        self.try_open()
    }
}
