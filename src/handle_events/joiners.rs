use std::io;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use crate::handle_events::StateChange;

impl crate::State {
    pub(super) fn poll_io_event(&mut self, timeout: Duration) -> io::Result<Option<StateChange>> {
        // SAFETY: we do not borrow self.joiners again
        let joiners = unsafe {
            std::mem::transmute::<&mut crate::state::Joiners, &mut crate::state::Joiners>(
                &mut self.joiners,
            )
        };

        joiners.runtime.block_on(async {
            tokio::select! {
                join_result_opt = joiners.read_dir_joiners.join_next() => {
                    let Some(join_result) = join_result_opt else {
                        return Ok(None);
                    };
                    let (path, entries) = join_result?;

                    self.handle_read_dir_event(path, entries);

                    Ok(Some(StateChange::ReEvalOpenedPath))
                }

                _ = tokio::time::sleep(timeout) => Ok(None)
            }
        })
    }

    fn handle_read_dir_event(&mut self, path: Arc<PathBuf>, entries: Vec<Arc<PathBuf>>) {
        let entry = crate::Entry::new_from_entries(path.clone(), entries, self.config.clone());
        self.entries.insert(path, entry);
    }
}
