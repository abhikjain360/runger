#[derive(Clone, Default)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub(crate) struct Typing {
    // TODO: use Cow
    input: String,
    completion: Option<Completion>,
}

#[derive(Clone)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub(crate) struct Completion {
    // TODO: use Cow
    candidates: Vec<String>,
    selected: Option<usize>,
}

impl Typing {
    pub(crate) fn new(input: String) -> Self {
        Self {
            input,
            completion: None,
        }
    }

    pub(crate) fn has_completion(&self) -> bool {
        self.completion.is_some()
    }

    pub(crate) fn visible_query(&self) -> &str {
        match &self.completion {
            Some(Completion {
                selected: Some(selected),
                candidates,
            }) => &candidates[*selected],
            _ => &self.input,
        }
    }

    pub(crate) fn cursor_pos(&self) -> u16 {
        match &self.completion {
            Some(Completion {
                candidates,
                selected: Some(selected),
                ..
            }) => candidates[*selected].len() as u16,
            _ => self.input.len() as u16,
        }
    }

    pub(crate) fn push_char(&mut self, c: char) {
        self.choose_selected_and_requery_completion();
        self.input.push(c);
    }

    pub(crate) fn pop_char(&mut self) {
        self.choose_selected_and_requery_completion();
        self.input.pop();
    }

    pub(crate) fn select_next_completion(&mut self) {
        let Some(Completion {
            candidates,
            selected: selected_opt,
        }) = self.completion.as_mut()
        else {
            return;
        };

        let Some(selected) = selected_opt else {
            *selected_opt = Some(0);
            return;
        };

        *selected += 1;
        if *selected >= candidates.len() {
            *selected_opt = None;
        }
    }

    pub(crate) fn select_prev_completion(&mut self) {
        let Some(Completion {
            candidates,
            selected: selected_opt,
        }) = self.completion.as_mut()
        else {
            return;
        };

        let Some(selected) = selected_opt else {
            *selected_opt = Some(candidates.len() - 1);
            return;
        };

        if *selected > 0 {
            *selected -= 1;
        } else {
            *selected_opt = None;
        }
    }

    pub(crate) fn select_completion(&mut self, next: bool) {
        if next {
            self.select_next_completion();
        } else {
            self.select_prev_completion();
        }
    }

    fn choose_selected_and_requery_completion(&mut self) {
        let Some(Completion {
            candidates,
            selected: Some(selected),
        }) = self.completion.take()
        else {
            return;
        };
        self.input = candidates[selected].clone();

        let candidates = candidates
            .into_iter()
            .filter(|candidate| candidate.starts_with(self.input.as_str()))
            .collect::<Vec<_>>();

        if !candidates.is_empty() {
            self.completion = Some(Completion {
                candidates,
                selected: None,
            });
        }
    }

    pub(crate) fn set_completion_candidates(&mut self, candidates: Vec<String>) {
        if candidates.is_empty() {
            self.completion.take();
            return;
        }

        self.completion = Some(Completion {
            candidates,
            selected: None,
        });
    }
}
