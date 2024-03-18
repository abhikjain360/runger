use std::{path::PathBuf, rc::Rc};

use crate::state::{Entry, State};

use super::entry::EntryType;

impl State {
    pub fn current_path(&self) -> CurrentPath {
        let next = self.first_visible_column.clone();
        CurrentPath {
            state: self,
            next: Some(next),
        }
    }

    pub(crate) fn current_path_mut(&mut self) -> CurrentPathMut {
        let next = self.first_visible_column.clone();
        CurrentPathMut {
            state: self,
            next: Some(next),
        }
    }
}

pub struct CurrentPath<'a> {
    pub(super) state: &'a State,
    next: Option<Rc<PathBuf>>,
}

impl<'a> Iterator for CurrentPath<'a> {
    type Item = &'a Entry;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.next.take()?;

        let entry = self.state.entry(&next)?;

        if let EntryType::Opened(opened) = &entry.ty {
            self.next = opened.selected_entry().cloned();
        };

        Some(entry)
    }
}

pub(crate) struct CurrentPathMut<'a> {
    pub(super) state: &'a mut State,
    next: Option<Rc<PathBuf>>,
}

impl<'a> Iterator for CurrentPathMut<'a> {
    type Item = &'a mut Entry;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.next.take()?;

        let entry = self.state.entry_mut(&next)?;

        if let EntryType::Opened(opened) = &entry.ty {
            self.next = opened.selected_entry().cloned();
        };

        // SAFETY: using transmute to change lifetime, which is fine because entry has lifetime of
        // 'a as it was taken from self.state which also has 'a lifetime
        Some(unsafe { std::mem::transmute(entry) })
    }
}
