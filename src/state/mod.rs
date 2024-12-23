use std::path::PathBuf;
use std::rc::Rc;

pub(crate) use crate::state::command::*;
pub(crate) use crate::state::command_palette::CommandPalette;
pub(crate) use crate::state::joiners::*;
use crate::{Entry, EntryType, Path};

mod command;
pub(crate) mod command_palette;
pub(crate) mod entry;
mod joiners;
mod visible_columns;

/// # Invariants
/// - assumes all paths all canonicalized and absolute
/// - `first_visible_column` exists in `entries`
/// - from `first_visible_column`, `selected_column` depth is valid
/// - `first_visible_column` is there in `entries`
//
// TODO: one of the ways to remove use of `unsafe` is to split `State` into multiple structs and
// then have functions take in more arguments. try this someday.
pub(crate) struct State {
    pub(crate) entries: crate::Map<Path, Entry>,
    pub(crate) first_visible_column: Path,
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

        let first_visible_column = Path::from(path.canonicalize()?);
        let first_entry = Entry::new(first_visible_column.clone(), None);

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

    fn create_entry_if_not_exists(
        &mut self,
        path: Path,
        select_on_open: Option<Path>,
    ) -> &mut Entry {
        self.entries
            .entry(path.clone())
            .or_insert(Entry::new(path.clone(), select_on_open))
    }

    /// Returns `true` if path is opened.
    pub(crate) fn try_open_selected_path(&mut self) -> bool {
        let required_depth = usize::from(self.config.required_columns) - self.selected_column;

        // SAFETY: we do not borrow self.joiners.read_dir_joiners again
        let joiner = unsafe {
            std::mem::transmute::<&mut ReadDirJoiner, &mut ReadDirJoiner>(
                &mut self.joiners.read_dir_joiner,
            )
        };
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

            self.create_entry_if_not_exists(next_path.clone(), None);

            #[cfg(debug_assertions)]
            {
                entry = self.entry_mut(next_path.as_ref()).unwrap();
            }

            #[cfg(not(debug_assertions))]
            {
                // SAFETY: we just inserted it in
                entry = unsafe { self.entry_mut(next_path.as_ref()).unwrap_unchecked() };
            }
        }

        true
    }

    /// Helper function for `move_right`. **DO NOT CALL DIRECTLY**, it might panic.
    fn get_next_visible_column(&mut self) -> Option<Path> {
        let first_visible_entry = match self.visible_columns_at(0) {
            Some(entry) => entry,
            None => {
                tracing::error!("no visible columns");
                return None;
            }
        };

        // move to right if opened and has a selected_entry
        match &first_visible_entry.ty {
            EntryType::Opened(opened) => {
                if let Some(next_path) = opened.selected_entry().cloned() {
                    self.create_entry_if_not_exists(next_path.clone(), None);
                    self.try_open_selected_path();

                    return Some(next_path);
                }

                tracing::error!("no selected entry even after State.entry_at_depth check");
                None
            }

            EntryType::Unopened(_) => {
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
        let parent_path = Path::from(parent_path.to_path_buf());

        // create an unopened entry for the parent path if there is none
        self.create_entry_if_not_exists(
            parent_path.clone(),
            Some(self.first_visible_column.clone()),
        );

        // try to open parent path
        self.first_visible_column = parent_path;
        self.try_open_selected_path();

        // TODO: find a way to set parent entry's selected_entry to child_path

        true
    }

    // TODO: support deleting multiple entries
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn delete_path(&mut self, path: Path) {
        let path = if path.is_absolute() {
            path
        } else {
            match path.as_ref().canonicalize() {
                Ok(path) => Path::from(path),
                Err(e) => {
                    tracing::error!("unable to canonicalize path = {path:?}: {e}");
                    return;
                }
            }
        };

        self.joiners.delete_joiner.spawn(path.clone());
        if !self.deleting_path_entry(path.clone()) {
            self.delete_path_entry_from_parent(&path);
        };
    }

    fn deleting_path_entry(&mut self, path: Path) -> bool {
        let Some(entry) = self.entries.get_mut(path.as_ref()) else {
            return false;
        };

        let entry = std::mem::replace(entry, Entry::deleting(path));

        if let EntryType::Opened(opened) = entry.ty {
            for entry in opened.entries {
                self.delete_path_entry(entry);
            }
        }

        true
    }

    pub(crate) fn delete_path_entry(&mut self, path: Path) {
        let Some(entry) = self.entries.swap_remove(path.as_ref()) else {
            return;
        };

        // delete the entry from parent, if exists
        self.delete_path_entry_from_parent(&entry.path);

        // if deleted entry is opened (that is, a directory), delete all its children as well
        if let EntryType::Opened(opened) = entry.ty {
            for entry in opened.entries {
                self.delete_path_entry(entry);
            }
        }
    }

    pub(crate) fn delete_path_entry_from_parent(&mut self, path: &Path) {
        if let Some(parent_entry) = path
            .parent()
            .and_then(|path| self.entries.get_mut(&path.to_path_buf()))
        {
            if let EntryType::Opened(opened) = &mut parent_entry.ty {
                if let Some(delete_idx) = opened
                    .entries
                    .iter()
                    .position(|e| e.as_ref() == path.as_ref())
                {
                    let deleted_path = opened.entries.remove(delete_idx);

                    // if after deleting the entry, the parent is empty, move left as we will
                    // render parent as empty dir
                    if opened.entries.is_empty() {
                        opened.selected = None;
                        self.move_left();
                    } else
                    // if deleted entry was selected, then set it to the next entry
                    if let Some(selected) = &opened.selected {
                        if selected.path() == &deleted_path {
                            // as 1 entry was deleted, delete_idx is already at next entry so we
                            // don't need to add 1
                            let mut next_idx = delete_idx;
                            // bound check
                            if next_idx >= opened.entries.len() {
                                next_idx = 0;
                            }

                            opened.set_selected(next_idx, 0);
                        }
                    } else {
                        tracing::warn!("opened entry which is not empty has no selected entry");
                    }
                } else {
                    tracing::warn!("parent entry exists but does not have the child entry");
                }
            } else {
                tracing::warn!("parent is not opened but still has children");
            }
        }
    }

    pub(crate) fn entry(&self, path: impl AsRef<std::path::Path>) -> Option<&Entry> {
        self.entries.get(path.as_ref())
    }

    pub(crate) fn first_entry(&self) -> &Entry {
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

    pub(crate) fn selected_entry(&self) -> &Entry {
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

    pub(crate) fn selected_entry_mut(&mut self) -> &mut Entry {
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

    fn entry_mut(&mut self, path: impl AsRef<std::path::Path>) -> Option<&mut Entry> {
        self.entries.get_mut(path.as_ref())
    }

    /// Returns the entry at the given depth, or the last entry with the depth reached if the depth
    /// provided is greater than the number of required columns. Follows the selected_entry at each
    /// column.
    ///
    /// Does not attempt to open any entries.
    pub(crate) fn entry_at_depth(&self, depth: usize) -> Result<&Entry, (&Entry, usize)> {
        let mut current_entry = self.first_entry();

        for current_depth in 1..=depth {
            match &current_entry.ty {
                EntryType::Opened(opened) => {
                    current_entry = opened
                        .selected_entry()
                        .and_then(|entry| self.entry(entry.as_ref()))
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
