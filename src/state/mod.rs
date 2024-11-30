use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

pub(crate) use joiners::*;

mod current_path;
pub(crate) mod entry;
mod joiners;
mod visible_columns;

/// # Invariants
/// - assumes all paths all canonicalized and absolute
/// - `first_visible_column` exists in `entries`
/// - from `first_visible_column`, `selected_column` depth is valid
pub(crate) struct State {
    pub(crate) entries: crate::Map<Arc<PathBuf>, crate::Entry>,
    pub(crate) first_visible_column: Arc<PathBuf>,
    pub(crate) config: Rc<crate::Config>,
    /// The column that is currently selected from the visible columns.
    pub(crate) selected_column: usize,
    pub(crate) joiners: Joiners,
}

impl State {
    pub fn new(path: PathBuf, config: crate::Config) -> crate::Result<Self> {
        let config = Rc::new(config);

        let first_visible_column = Arc::new(path.canonicalize()?);
        let first_entry = crate::Entry::new(first_visible_column.clone(), config.clone());

        let entries =
            crate::Map::from_iter(std::iter::once((first_visible_column.clone(), first_entry)));

        let joiners = Joiners::new()?;

        let mut ret = Self {
            entries,
            first_visible_column,
            config,
            selected_column: 0,
            joiners,
        };

        ret.try_open_selected_path()?;

        Ok(ret)
    }

    fn create_entry_if_not_exists(&mut self, path: &Arc<PathBuf>) {
        if self.entry(path).is_none() {
            let next_entry = crate::Entry::new(path.clone(), self.config.clone());
            self.entries.insert(path.clone(), next_entry);
        }
    }

    pub(crate) fn try_open_selected_path(&mut self) -> crate::Result<()> {
        let required_depth = usize::from(self.config.required_columns) - self.selected_column;
        let mut entry = self.selected_entry_mut();

        for _ in 0..required_depth {
            let Some(next_path) = entry.try_open()?.and_then(|opened| opened.selected_entry())
            else {
                break;
            };
            let next_path = next_path.clone();

            self.create_entry_if_not_exists(&next_path);

            // SAFETY: we just inserted it in
            entry = self.entry_mut(next_path).unwrap();
        }

        Ok(())
    }

    pub fn move_right(&mut self) -> crate::Result<bool> {
        let current_path = self.current_path().collect::<Vec<_>>();

        if current_path.len() > self.config.required_columns.into() {
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
            crate::EntryType::File if self.selected_column + 2 >= current_path.len() => {
                // we can not expand further as the current_path is at its end, as well as
                // selected_column is also at the end of current_path
                Ok(false)
            }

            crate::EntryType::File => {
                // though we can not shift the visible_columns further left as we
                // current_path.last == File, we can still move the selected_column one further
                // as current_path has more to show
                self.selected_column += 1;
                Ok(true)
            }

            crate::EntryType::Opened(ref opened) if opened.entries.is_empty() => {
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

    pub fn move_left(&mut self) -> crate::Result<bool> {
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
        let parent_path = Arc::new(parent_path.to_path_buf());

        // create an unopened entry for the parent path if there is none
        self.create_entry_if_not_exists(&parent_path);

        // try to open parent path
        self.first_visible_column = parent_path;
        self.try_open_selected_path()?;

        if let crate::Entry {
            ty: crate::EntryType::Opened(opened),
            ..
        } = self.selected_entry_mut()
        {
            if !opened.set_selected_entry(&child_path) {
                tracing::warn!("unable to set the selected entry in parent column");
            }
        }

        Ok(true)
    }

    pub fn entry(&self, path: impl AsRef<PathBuf>) -> Option<&crate::Entry> {
        self.entries.get(path.as_ref())
    }

    #[expect(dead_code)]
    pub fn first_entry(&self) -> &crate::Entry {
        self.entries
            .get(&self.first_visible_column)
            .expect("self.first_visible_column does not exist")
    }

    #[expect(dead_code)]
    pub(crate) fn selected_entry(&self) -> &crate::Entry {
        self.visible_columns_at(self.selected_column)
            .expect("self.selected_column does not exist")
    }

    pub(crate) fn selected_entry_mut(&mut self) -> &mut crate::Entry {
        self.visible_columns_mut_at(self.selected_column)
            .expect("self.selected_column does not exist")
    }

    fn entry_mut(&mut self, path: impl AsRef<PathBuf>) -> Option<&mut crate::Entry> {
        self.entries.get_mut(path.as_ref())
    }
}
