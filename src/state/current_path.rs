use std::path::PathBuf;
use std::sync::Arc;

impl crate::State {
    pub fn current_path(&self) -> CurrentPath {
        let next = self.first_visible_column.clone();
        CurrentPath {
            state: self,
            next: Some(next),
        }
    }
}

pub struct CurrentPath<'a> {
    pub(super) state: &'a crate::State,
    next: Option<Arc<PathBuf>>,
}

impl<'a> Iterator for CurrentPath<'a> {
    type Item = &'a crate::Entry;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.next.take()?;

        let entry = self.state.entry(&next)?;

        if let crate::EntryType::Opened(opened) = &entry.ty {
            self.next = opened.selected_entry().cloned();
        };

        Some(entry)
    }
}
