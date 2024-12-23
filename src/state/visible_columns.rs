use crate::{Entry, EntryType, Path, State};

impl State {
    pub(crate) fn visible_columns_mut(&mut self) -> VisibleColumnsMut {
        let first_column = self.first_visible_column.clone();
        VisibleColumnsMut {
            state: self,
            next: Some(first_column),
            depth: 0,
        }
    }

    pub(super) fn visible_columns_at(&self, depth: usize) -> Option<&Entry> {
        if depth > self.config.required_columns.into() {
            return None;
        }

        let mut current_path = self.first_visible_column.clone();
        for _ in 1..=depth {
            let entry = self.entry(current_path.as_ref())?;

            current_path = match &entry.ty {
                EntryType::Opened(opened) => opened.selected_entry()?.clone(),
                _ => return None,
            };
        }

        self.entry(current_path.as_ref())
    }

    pub(super) fn visible_columns_mut_at(&mut self, depth: usize) -> Option<&mut Entry> {
        if depth > self.config.required_columns.into() {
            return None;
        }

        let mut current_path = self.first_visible_column.clone();
        for _ in 1..=depth {
            let entry = self.entry_mut(current_path.as_ref())?;

            current_path = match &entry.ty {
                EntryType::Opened(opened) => opened.selected_entry()?.clone(),
                _ => return None,
            };
        }

        self.entry_mut(current_path.as_ref())
    }
}

pub(crate) struct VisibleColumnsMut<'a> {
    state: &'a mut State,
    next: Option<Path>,
    depth: usize,
}

impl<'a> Iterator for VisibleColumnsMut<'a> {
    type Item = &'a mut Entry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.depth == self.state.config.required_columns.into() {
            return None;
        }

        let next = self.next.take()?;

        let entry = self.state.entry_mut(next.as_ref())?;

        if let EntryType::Opened(opened) = &entry.ty {
            self.next = opened.selected_entry().cloned();
        };

        self.depth += 1;

        // SAFETY: using transmute to change lifetime, which is fine because entry has lifetime of
        // 'a as it was taken from self.state which also has 'a lifetime
        Some(unsafe { std::mem::transmute::<&mut Entry, &'a mut Entry>(entry) })
    }
}
