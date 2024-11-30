use std::fs;
use std::io;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

use crate::{
    config::Config,
    state::entry::opened::{OpenedEntries, Selected},
    Result,
};

pub mod opened;
pub use opened::Opened;

pub(crate) enum EntryType {
    Opened(Opened),
    File,
    Unopened,
}

pub struct Entry {
    pub(crate) path: Arc<PathBuf>,
    pub(crate) ty: EntryType,
    pub(crate) config: Rc<Config>,
}

pub(crate) enum TryOpenResult<'a> {
    File,
    Opened(&'a Opened),
    Waiting,
}

impl Entry {
    pub(crate) fn new(path: Arc<PathBuf>, config: Rc<Config>) -> Self {
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
            .map(|entry_res| Ok(Arc::new(entry_res?.path().to_path_buf())))
            .collect::<Result<Vec<_>>>()?;
        entries.sort();

        self.ty = EntryType::Opened(Opened {
            selected: entries.first().map(|_| Selected::new(0, 0)),
            entries: OpenedEntries::Entries(entries),
            config: self.config.clone(),
        });

        self.try_open()
    }

    #[allow(dead_code)]
    pub(crate) fn try_open_async(
        &mut self,
        joiner: &mut crate::state::ReadDirJoiner,
    ) -> TryOpenResult<'_> {
        match self.ty {
            EntryType::File => return TryOpenResult::File,
            EntryType::Opened(ref mut opened) => return TryOpenResult::Opened(opened),
            EntryType::Unopened => {}
        };

        joiner.spawn(self.path.clone());

        TryOpenResult::Waiting
    }
}
