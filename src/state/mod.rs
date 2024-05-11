#![allow(dead_code)]
use std::{num::NonZeroUsize, path::PathBuf, rc::Rc};

pub(crate) use crate::state::entry::Entry;
use crate::{Result, RungerMap};

use self::entry::EntryType;

mod current_path;
pub(crate) mod entry;
mod visible_columns;

/// # Invariants
/// - assumes all paths all canonicalized and absolute
/// - `first_visible_column` exists in `entries`
/// - from `first_visible_column`, `selected_column` depth is valid
#[derive(Debug, Clone)]
pub struct State {
    pub(crate) entries: RungerMap<Rc<PathBuf>, Entry>,
    pub(crate) first_visible_column: Rc<PathBuf>,
    pub(crate) required_columns: NonZeroUsize,
    pub(crate) selected_column: usize,
}

impl State {
    pub fn new(path: PathBuf, required_columns: NonZeroUsize) -> Result<Self> {
        let first_visible_column = Rc::new(path.canonicalize()?);
        let first_entry = Entry::new(first_visible_column.clone());
        let mut ret = Self {
            entries: RungerMap::from_iter(std::iter::once((
                first_visible_column.clone(),
                first_entry,
            ))),
            first_visible_column,
            required_columns,
            selected_column: 0,
        };

        ret.try_open_selected_path()?;

        Ok(ret)
    }

    fn create_entry_if_not_exists(&mut self, path: &Rc<PathBuf>) {
        if self.entry(path).is_none() {
            let next_entry = Entry::new(path.clone());
            self.entries.insert(path.clone(), next_entry);
        }
    }

    pub(crate) fn try_open_selected_path(&mut self) -> Result<()> {
        let mut required_depth = usize::from(self.required_columns) - self.selected_column;
        let mut entry = self.selected_entry_mut();

        while required_depth > 0 {
            let Some(next_path) = entry.try_open()?.and_then(|opened| opened.selected_entry())
            else {
                break;
            };
            let next_path = next_path.clone();

            self.create_entry_if_not_exists(&next_path);

            // SAFETY: we just inserted it in
            entry = self.entry_mut(next_path).unwrap();

            required_depth -= 1;
        }

        Ok(())
    }

    pub fn move_right(&mut self) -> Result<bool> {
        let current_path = self.current_path().collect::<Vec<_>>();

        if current_path.len() > self.required_columns.into() {
            // current_path has more room then required_columns, so move the visible_columns
            // forward
            self.first_visible_column = current_path[1].path.clone();
            return Ok(true);
        }

        // current path ran out of things to show for all the required_columns, show we try to:
        //   1. extend the current path, or
        //   2. move the selected_column
        // in that order
        //
        // PANIC SAFETY: current_path should at least have self.first_visible_column
        match current_path.last().unwrap().ty {
            EntryType::File if self.selected_column + 2 >= current_path.len() => {
                // we can not expand further as the current_path is at its end, as well as
                // selected_column is also at the end of current_path
                Ok(false)
            }

            EntryType::File => {
                // though we can not shift the visible_columns further left as we
                // current_path.last == File, we can still move the selected_column one further
                // as current_path has more to show
                self.selected_column += 1;
                Ok(true)
            }

            EntryType::Opened(ref opened) if opened.entries.is_empty() => {
                // we can not open further as the directory is empty, but we can check to see if we
                // can move selected_column further

                if self.selected_column + 2 >= current_path.len() {
                    return Ok(false);
                }

                self.selected_column += 1;
                Ok(true)
            }

            _ => {
                // current_path.len < required_columns and current_path.last is either:
                // - unopened dir, or
                // - opened dir which is not empty
                // so we can try opening the selected path further and retry
                self.try_open_selected_path()?;
                self.move_right()
            }
        }
    }

    pub fn move_left(&mut self) -> Result<bool> {
        // try to just move selected_column
        if self.selected_column > 0 {
            self.selected_column -= 1;
            return Ok(true);
        }
        // otherwise move the visible_columns back

        let Some(parent_path) = self.first_visible_column.parent() else {
            // parent_path does not exist
            return Ok(false);
        };
        let child_path = self.first_visible_column.clone();
        let parent_path = Rc::new(parent_path.to_path_buf());

        // create an unopened entry for the parent path if there is none
        self.create_entry_if_not_exists(&parent_path);

        // try to open parent path
        self.first_visible_column = parent_path;
        self.try_open_selected_path()?;

        if let Entry {
            ty: EntryType::Opened(opened),
            ..
        } = self.selected_entry_mut()
        {
            if !opened.set_selected_entry(&child_path) {
                tracing::warn!("unable to set the selected entry in parent column");
            }
        }

        Ok(true)
    }

    pub fn entry(&self, path: impl AsRef<PathBuf>) -> Option<&Entry> {
        self.entries.get(path.as_ref())
    }

    pub fn first_entry(&self) -> &Entry {
        self.entries
            .get(&self.first_visible_column)
            .expect("self.first_visible_column does not exist")
    }

    pub(crate) fn selected_entry(&self) -> &Entry {
        self.visible_columns()
            .nth(self.selected_column)
            .expect("self.selected_column does not exist")
    }

    pub(crate) fn selected_entry_mut(&mut self) -> &mut Entry {
        let selected_column = self.selected_column;
        self.visible_columns_mut()
            .nth(selected_column)
            .expect("self.selected_column does not exist")
    }

    fn entry_mut(&mut self, path: impl AsRef<PathBuf>) -> Option<&mut Entry> {
        self.entries.get_mut(path.as_ref())
    }
}
