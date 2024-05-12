use std::{path::PathBuf, rc::Rc};

use crate::state::{Entry, EntryType, State};

impl State {
    pub fn visible_columns(&self) -> VisibleColumns {
        VisibleColumns {
            state: self,
            next: Some(self.first_visible_column.clone()),
            depth: 0,
        }
    }

    pub(crate) fn visible_columns_mut(&mut self) -> VisibleColumnsMut {
        let first_column = self.first_visible_column.clone();
        VisibleColumnsMut {
            state: self,
            next: Some(first_column),
            depth: 0,
        }
    }
}

pub struct VisibleColumns<'a> {
    state: &'a State,
    next: Option<Rc<PathBuf>>,
    depth: usize,
}

impl<'a> Iterator for VisibleColumns<'a> {
    type Item = &'a Entry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.depth == self.state.config.required_columns.into() {
            return None;
        }

        let next = self.next.take()?;

        let entry = self.state.entry(&next)?;

        if let EntryType::Opened(opened) = &entry.ty {
            self.next = opened.selected_entry().cloned();
        };

        self.depth += 1;
        Some(entry)
    }
}

pub(crate) struct VisibleColumnsMut<'a> {
    state: &'a mut State,
    next: Option<Rc<PathBuf>>,
    depth: usize,
}

impl<'a> Iterator for VisibleColumnsMut<'a> {
    type Item = &'a mut Entry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.depth == self.state.config.required_columns.into() {
            return None;
        }

        let next = self.next.take()?;

        let entry = self.state.entry_mut(&next)?;

        if let EntryType::Opened(opened) = &entry.ty {
            self.next = opened.selected_entry().cloned();
        };

        self.depth += 1;

        // SAFETY: using transmute to change lifetime, which is fine because entry has lifetime of
        // 'a as it was taken from self.state which also has 'a lifetime
        Some(unsafe { std::mem::transmute(entry) })
    }
}
