use std::io;
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

                    self.handle_read_dir_event(join_result)?;

                    Ok(Some(StateChange::ReEvalOpenedPath))
                }

                _ = tokio::time::sleep(timeout) => Ok(None)
            }
        })
    }

    fn handle_read_dir_event(&mut self, result: crate::state::ReadDirResult) -> io::Result<()> {
        let entry = match result.kind {
            crate::state::ReadDirResultKind::PermissionDenied => {
                crate::Entry::permission_denied(result.path.clone())
            }
            crate::state::ReadDirResultKind::Ok(entries) => {
                crate::Entry::opened(result.path.clone(), entries, self.config.clone())
            }
            crate::state::ReadDirResultKind::Err(e) => return Err(e),
        };

        self.entries.insert(result.path, entry);
        Ok(())
    }
}
