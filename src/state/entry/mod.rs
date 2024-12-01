#![allow(unused_imports)]
use std::fs;
use std::io;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

use crate::{config::Config, Result};

pub mod opened;
pub use opened::{Opened, Selected};

#[cfg_attr(debug_assertions, derive(Debug))]
pub(crate) enum EntryType {
    Opened(Opened),
    File,
    Unopened,
    Waiting,
    PermissionDenied,
}

#[cfg_attr(debug_assertions, derive(Debug))]
pub struct Entry {
    pub(crate) path: Arc<PathBuf>,
    pub(crate) ty: EntryType,
}

pub(crate) enum TryOpen<T> {
    File,
    Opened(T),
    Waiting,
    PermissionDenied,
}

impl Entry {
    pub(crate) fn new(path: Arc<PathBuf>) -> Self {
        let mut ret = Self {
            path: path.clone(),
            ty: EntryType::File,
        };
        if path.is_dir() {
            ret.ty = EntryType::Unopened;
        }
        ret
    }

    // pub(crate) fn try_open(&mut self) -> Result<Option<&Opened>> {
    //     match self.ty {
    //         EntryType::File => return Ok(None),
    //         EntryType::Opened(ref mut opened) => return Ok(Some(opened)),
    //         EntryType::Unopened => {}
    //     };
    //
    //     let entries = match fs::read_dir(self.path.as_ref()) {
    //         Err(e) if e.kind() == io::ErrorKind::PermissionDenied => return Ok(None),
    //         res => res?,
    //     };
    //
    //     let mut entries = entries
    //         .map(|entry_res| Ok(Arc::new(entry_res?.path().to_path_buf())))
    //         .collect::<Result<Vec<_>>>()?;
    //     entries.sort();
    //
    //     self.ty = EntryType::Opened(Opened {
    //         selected: entries.first().map(|_| Selected::new(0, 0)),
    //         entries: OpenedEntries::Entries(entries),
    //         config: self.config.clone(),
    //     });
    //
    //     self.try_open()
    // }

    pub(crate) fn try_open(
        &mut self,
        joiner: &mut crate::state::ReadDirJoiner,
    ) -> TryOpen<&'_ Opened> {
        match self.ty {
            EntryType::File => return TryOpen::File,
            EntryType::PermissionDenied => return TryOpen::PermissionDenied,
            EntryType::Opened(ref mut opened) => return TryOpen::Opened(opened),
            EntryType::Waiting => {}
            EntryType::Unopened => {}
        };

        joiner.spawn(self.path.clone());
        tracing::debug!("spawned read_dir for {:?}", self.path);
        self.ty = EntryType::Waiting;

        TryOpen::Waiting
    }

    pub(crate) fn opened(
        path: Arc<PathBuf>,
        mut entries: Vec<Arc<PathBuf>>,
        config: Rc<Config>,
    ) -> Self {
        entries.sort();

        let ty = EntryType::Opened(Opened {
            selected: entries.first().map(|_| Selected::new(0, 0)),
            entries,
            config: config.clone(),
        });

        Self { path, ty }
    }

    pub(crate) fn permission_denied(path: Arc<PathBuf>) -> Self {
        Self {
            path,
            ty: EntryType::PermissionDenied,
        }
    }
}
