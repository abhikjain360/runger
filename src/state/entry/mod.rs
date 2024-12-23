#![allow(unused_imports)]
use std::fs;
use std::io;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

use crate::{config::Config, Result};
pub(crate) use opened::{Opened, Selected};
use unopened::Unopened;

mod opened;
mod unopened;

#[cfg_attr(debug_assertions, derive(Debug))]
pub(crate) enum EntryType {
    Opened(Opened),
    File,
    Unopened(Unopened),
    Deleting,
    Waiting(Unopened),
    PermissionDenied,
}

#[cfg_attr(debug_assertions, derive(Debug))]
pub struct Entry {
    pub(crate) path: Arc<PathBuf>,
    pub(crate) ty: EntryType,
}

impl Entry {
    pub(crate) fn new(path: Arc<PathBuf>, select_on_open: Option<Arc<PathBuf>>) -> Self {
        let mut ret = Self {
            path: path.clone(),
            ty: EntryType::File,
        };
        if path.is_dir() {
            ret.ty = EntryType::Unopened(Unopened { select_on_open });
        }
        ret
    }

    #[tracing::instrument(level = "trace", skip(self, joiner))]
    pub(crate) fn try_open(&mut self, joiner: &mut crate::state::ReadDirJoiner) {
        if let EntryType::Unopened(unopened) = std::mem::replace(&mut self.ty, EntryType::Deleting)
        {
            joiner.spawn(self.path.clone());
            self.ty = EntryType::Waiting(unopened);
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
        select_on_open: Option<Arc<PathBuf>>,
    ) -> Self {
        entries.sort();

        let selected = select_on_open
            .and_then(|selected_path| {
                entries
                    .binary_search(&selected_path)
                    .ok()
                    .map(|_| selected_path)
            })
            .or_else(|| entries.first().cloned())
            .map(|selected_path| Selected::new(selected_path, 0));

        let ty = EntryType::Opened(Opened {
            selected,
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
        matches!(self.ty, EntryType::Unopened(_))
    }

    #[expect(dead_code)]
    pub(crate) fn is_opened(&self) -> bool {
        matches!(self.ty, EntryType::Opened(_))
    }
}
