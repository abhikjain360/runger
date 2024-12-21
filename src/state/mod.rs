use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

pub(crate) use crate::state::command::*;
pub(crate) use crate::state::command_palette::CommandPalette;
pub(crate) use crate::state::joiners::*;

mod command;
mod command_palette;
pub(crate) mod entry;
mod joiners;
mod visible_columns;

/// # Invariants
/// - assumes all paths all canonicalized and absolute
/// - `first_visible_column` exists in `entries`
/// - from `first_visible_column`, `selected_column` depth is valid
/// - `first_visible_column` is there in `entries`
pub(crate) struct State {
    pub(crate) entries: crate::Map<Arc<PathBuf>, crate::Entry>,
    pub(crate) first_visible_column: Arc<PathBuf>,
    pub(crate) config: Rc<crate::Config>,
    /// The column that is currently selected from the visible columns.
    pub(crate) selected_column: usize,
    pub(crate) joiners: Joiners,
    pub(crate) command_palette: CommandPalette,
    pub(crate) command_palette_row: u16,
}

impl State {
    pub(crate) fn new(path: PathBuf, config: crate::Config) -> crate::Result<Self> {
        let config = Rc::new(config);

        let first_visible_column = Arc::new(path.canonicalize()?);
        let first_entry = crate::Entry::new(first_visible_column.clone());

        let entries =
            crate::Map::from_iter(std::iter::once((first_visible_column.clone(), first_entry)));

        let joiners = Joiners::new()?;

        let mut ret = Self {
            entries,
            first_visible_column,
            config,
            selected_column: 0,
            joiners,
            command_palette: CommandPalette::Empty,
            command_palette_row: 0,
        };

        ret.try_open_selected_path();

        Ok(ret)
    }

    fn create_entry_if_not_exists(&mut self, path: &Arc<PathBuf>) -> &mut crate::Entry {
        self.entries
            .entry(path.clone())
            .or_insert(crate::Entry::new(path.clone()))
    }

    /// Returns `true` if path is opened.
    pub(crate) fn try_open_selected_path(&mut self) -> bool {
        let required_depth = usize::from(self.config.required_columns) - self.selected_column;

        // SAFETY: we do not borrow self.joiners.read_dir_joiners again
        let joiner =
            unsafe { std::mem::transmute::<&mut _, &mut _>(&mut self.joiners.read_dir_joiner) };
        let mut entry = self.selected_entry_mut();

        for _ in 0..required_depth {
            if entry.is_unopened() {
                entry.try_open(joiner);
                return false;
            }

            let Some(next_path) = entry
                .get_opened()
                .and_then(|opened| opened.selected_entry())
                .cloned()
            else {
                return false;
            };

            self.create_entry_if_not_exists(&next_path);

            #[cfg(debug_assertions)]
            {
                entry = self.entry_mut(next_path).unwrap();
            }

            #[cfg(not(debug_assertions))]
            {
                // SAFETY: we just inserted it in
                entry = unsafe { self.entry_mut(next_path).unwrap_unchecked() };
            }
        }

        true
    }

    /// Helper function for `move_right`. **DO NOT CALL DIRECTLY**, it might panic.
    // TODO: move this to a separate crate so that it can not be called directly
    fn get_next_visible_column(&mut self) -> Option<Arc<PathBuf>> {
        let first_visible_entry = match self.visible_columns_at(0) {
            Some(entry) => entry,
            None => {
                tracing::error!("no visible columns");
                return None;
            }
        };

        // move to right if opened and has a selected_entry
        match &first_visible_entry.ty {
            crate::EntryType::Opened(opened) => {
                if let Some(next_path) = opened.selected_entry().cloned() {
                    self.create_entry_if_not_exists(&next_path);
                    self.try_open_selected_path();

                    return Some(next_path);
                }

                tracing::error!("no selected entry even after State.entry_at_depth check");
                None
            }

            crate::EntryType::Unopened => {
                tracing::error!("encountered unopened entry even after State.entry_at_depth check");
                let path = first_visible_entry.path.clone();
                self.joiners.read_dir_joiner.spawn(path);
                None
            }

            _ => {
                tracing::error!(
                    "encountered !opened entry type even after State.entry_at_depth check"
                );
                None
            }
        }
    }

    /// Returns true if we moved right.
    pub(crate) fn move_right(&mut self) -> bool {
        match self.entry_at_depth(self.required_columns()) {
            Ok(_) => {
                if self.selected_column + 2 < self.required_columns() {
                    self.selected_column += 1;
                    return true;
                }

                if let Some(next_start) = self.get_next_visible_column() {
                    self.first_visible_column = next_start;
                    return true;
                }

                false
            }
            Err((_, depth)) if depth > self.selected_column + 2 => {
                self.try_open_selected_path();
                self.selected_column += 1;
                true
            }
            Err(_) => {
                self.try_open_selected_path();
                false
            }
        }

        // match self.entry_at_depth(required_columns) {
        //     // current path has more columns than required columns and the last entry is opened, so
        //     // we can shift the visible columns.
        //     Ok(entry) if entry.is_opened() => {
        //         if let Some(next_start) = self.move_right_inner() {
        //             self.first_visible_column = next_start;
        //             return true;
        //         };
        //         false
        //     }
        //
        //     // current path has more columns than required columns and the last entry is unopened,
        //     // so we try to open the selected path but can not shift the visible columns. meanwhile
        //     // we move the selected_column.
        //     Ok(entry) if entry.is_unopened() => {
        //         self.try_open_selected_path();
        //         self.try_move_selected_column_right(required_columns)
        //     }
        //
        //     // current path has more columns than required columns and the last entry is neither
        //     // opened nor unopened, so we can not shift the visible columns. we just move the
        //     // selected_column.
        //     // TODO: handle the case when entry is still pending
        //     Ok(_) => self.try_move_selected_column_right(required_columns),
        //
        //     // current path has less columns than required columns, so we can not shift the visible
        //     // columns. we just move the selected_column.
        //     Err((_, depth)) if depth > self.selected_column + 2 => {
        //         self.try_move_selected_column_right(required_columns)
        //     }
        //
        //     // current path has less columns than required columns, so we can not shift the visible
        //     // columns. also depth is less than selected_column + 2, so we can not move the
        //     // selected column either.
        //     Err(_) => false,
        // }
    }

    pub(crate) fn move_left(&mut self) -> bool {
        // try to just move selected_column
        if self.selected_column > 0 {
            self.selected_column -= 1;
            return true;
        }
        // otherwise move the visible_columns back

        let Some(parent_path) = self.first_visible_column.parent() else {
            // parent_path does not exist
            return false;
        };
        let parent_path = Arc::new(parent_path.to_path_buf());

        // create an unopened entry for the parent path if there is none
        self.create_entry_if_not_exists(&parent_path);

        // try to open parent path
        self.first_visible_column = parent_path;
        self.try_open_selected_path();

        // TODO: find a way to set parent entry's selected_entry to child_path

        true
    }

    // TODO: support deleting multiple entries
    pub(crate) fn delete_path(&mut self, path: impl AsRef<PathBuf>) {
        self.joiners
            .delete_joiner
            .spawn(Arc::new(path.as_ref().to_path_buf()));

        self.delete_path_entry(path);
    }

    fn delete_path_entry(&mut self, path: impl AsRef<PathBuf>) {
        let Some(entry) = self.entries.swap_remove(path.as_ref()) else {
            return;
        };

        if let Some(entry) = entry
            .path
            .parent()
            .and_then(|path| self.entries.get_mut(&path.to_path_buf()))
        {
            if let crate::EntryType::Opened(opened) = &mut entry.ty {
                if let Some(idx) = opened
                    .entries
                    .iter()
                    .position(|e| e.as_ref() == path.as_ref())
                {
                    opened.entries.remove(idx);
                } else {
                    tracing::warn!("parent entry exists but does not have the child entry");
                }
            }
        }

        if let crate::EntryType::Opened(opened) = entry.ty {
            for entry in opened.entries {
                self.delete_path_entry(entry);
            }
        }
    }

    pub(crate) fn entry(&self, path: impl AsRef<PathBuf>) -> Option<&crate::Entry> {
        self.entries.get(path.as_ref())
    }

    pub(crate) fn first_entry(&self) -> &crate::Entry {
        let entry = self.entries.get(&self.first_visible_column);

        #[cfg(debug_assertions)]
        {
            entry.expect("self.first_visible_column does not exist")
        }

        #[cfg(not(debug_assertions))]
        unsafe {
            // SAFETY: self.first_visible_column is always there
            entry.unwrap_unchecked()
        }
    }

    pub(crate) fn selected_entry(&self) -> &crate::Entry {
        #[cfg(debug_assertions)]
        {
            self.visible_columns_at(self.selected_column)
                .expect("self.selected_column does not exist")
        }

        #[cfg(not(debug_assertions))]
        unsafe {
            // SAFETY: self.selected_column is always valid
            self.visible_columns_at(self.selected_column)
                .unwrap_unchecked()
        }
    }

    pub(crate) fn selected_entry_mut(&mut self) -> &mut crate::Entry {
        #[cfg(debug_assertions)]
        {
            self.visible_columns_mut_at(self.selected_column)
                .expect("self.selected_column does not exist")
        }

        #[cfg(not(debug_assertions))]
        unsafe {
            // SAFETY: self.selected_column is always valid
            self.visible_columns_mut_at(self.selected_column)
                .unwrap_unchecked()
        }
    }

    fn entry_mut(&mut self, path: impl AsRef<PathBuf>) -> Option<&mut crate::Entry> {
        self.entries.get_mut(path.as_ref())
    }

    /// Returns the entry at the given depth, or the last entry with the depth reached if the depth
    /// provided is greater than the number of required columns. Follows the selected_entry at each
    /// column.
    ///
    /// Does not attempt to open any entries.
    pub(crate) fn entry_at_depth(
        &self,
        depth: usize,
    ) -> Result<&crate::Entry, (&crate::Entry, usize)> {
        let mut current_entry = self.first_entry();

        for current_depth in 1..=depth {
            match &current_entry.ty {
                crate::EntryType::Opened(opened) => {
                    current_entry = opened
                        .selected_entry()
                        .and_then(|entry| self.entry(entry))
                        .ok_or((current_entry, current_depth))?;
                }
                _ => return Err((current_entry, current_depth)),
            }
        }

        Ok(current_entry)
    }

    pub(crate) fn required_columns(&self) -> usize {
        self.config.required_columns.get()
    }
}
