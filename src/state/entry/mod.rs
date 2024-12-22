#![allow(unused_imports)]
use std::fs;
use std::io;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

use crate::{config::Config, Result};

pub(crate) mod opened;
pub(crate) use opened::{Opened, Selected};

#[cfg_attr(debug_assertions, derive(Debug))]
pub(crate) enum EntryType {
    Opened(Opened),
    File,
    Unopened,
    Deleting,
    Waiting,
    PermissionDenied,
}

#[cfg_attr(debug_assertions, derive(Debug))]
pub struct Entry {
    pub(crate) path: Arc<PathBuf>,
    pub(crate) ty: EntryType,
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

    #[tracing::instrument(level = "trace", skip(self, joiner))]
    pub(crate) fn try_open(&mut self, joiner: &mut crate::state::ReadDirJoiner) {
        if self.is_unopened() {
            joiner.spawn(self.path.clone());
            self.ty = EntryType::Waiting;
        }
    }

    pub(crate) fn get_opened(&self) -> Option<&Opened> {
        match &self.ty {
            EntryType::Opened(opened) => Some(opened),
            _ => None,
        }
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

    pub(crate) fn deleting(path: Arc<PathBuf>) -> Self {
        Self {
            path,
            ty: EntryType::Deleting,
        }
    }

    pub(crate) fn permission_denied(path: Arc<PathBuf>) -> Self {
        Self {
            path,
            ty: EntryType::PermissionDenied,
        }
    }

    pub(crate) fn file(path: Arc<PathBuf>) -> Self {
        Self {
            path,
            ty: EntryType::File,
        }
    }

    pub(crate) fn is_unopened(&self) -> bool {
        matches!(self.ty, EntryType::Unopened)
    }

    #[expect(dead_code)]
    pub(crate) fn is_opened(&self) -> bool {
        matches!(self.ty, EntryType::Opened(_))
    }
}
