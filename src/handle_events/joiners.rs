use std::io;
use std::time::Duration;

use crate::handle_events::StateChange;

impl crate::State {
    pub(super) fn poll_io_event(&mut self, timeout: Duration) -> io::Result<Option<StateChange>> {
        let joiners = &mut self.joiners;
        joiners.runtime.block_on(async {
            tokio::select! {
                _ = joiners.read_dir_joiners.join_next() => {}
                _ = tokio::time::sleep(timeout) => {}
            }
        });
        Ok(None)
    }
}
