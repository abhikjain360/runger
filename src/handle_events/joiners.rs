use std::io;
use std::time::Duration;

use futures::stream::FuturesUnordered;
use futures::{FutureExt, StreamExt};

use crate::handle_events::StateChange;
use crate::state::ReadDirResult;

impl crate::State {
    pub(super) fn poll_io_event(&mut self, timeout: Duration) -> io::Result<Option<StateChange>> {
        enum PollResult {
            Delete(io::Result<()>),
            ReadDir(ReadDirResult),
            Timeout,
        }

        // SAFETY: we do not borrow self.joiners again
        let joiners = unsafe {
            std::mem::transmute::<&mut crate::state::Joiners, &mut crate::state::Joiners>(
                &mut self.joiners,
            )
        };

        let mut futures = FuturesUnordered::new();

        if !joiners.delete_joiner.is_empty() {
            futures.push(
                async {
                    joiners
                        .delete_joiner
                        .join_next()
                        .await
                        .map(PollResult::Delete)
                }
                .boxed(),
            );
        }

        if !joiners.read_dir_joiner.is_empty() {
            futures.push(
                async {
                    joiners
                        .read_dir_joiner
                        .join_next()
                        .await
                        .map(PollResult::ReadDir)
                }
                .boxed(),
            );
        }

        if futures.is_empty() {
            return Ok(None);
        }

        futures.push(
            async {
                tokio::time::sleep(timeout).await;
                Some(PollResult::Timeout)
            }
            .boxed(),
        );

        let res = joiners.runtime.block_on(async {
            futures.next().await.ok_or(io::Error::new(
                io::ErrorKind::TimedOut,
                "timeout reached while waiting for IO events",
            ))
        })?;

        let Some(res) = res else {
            return Ok(None);
        };

        match res {
            PollResult::Delete(res) => {
                res?;
                Ok(Some(StateChange::NoActionRequired))
            }
            PollResult::ReadDir(res) => {
                self.handle_read_dir_event(res)?;
                self.try_open_selected_path();
                Ok(Some(StateChange::NoActionRequired))
            }
            PollResult::Timeout => Ok(None),
        }
    }

    fn handle_read_dir_event(&mut self, result: crate::state::ReadDirResult) -> io::Result<()> {
        let entry = match result.kind {
            crate::state::ReadDirResultKind::PermissionDenied => {
                crate::Entry::permission_denied(result.path.clone())
            }
            crate::state::ReadDirResultKind::NotADirectory => {
                crate::Entry::file(result.path.clone())
            }
            crate::state::ReadDirResultKind::Ok(entries) => {
                crate::Entry::opened(result.path.clone(), entries, self.config.clone())
            }
            crate::state::ReadDirResultKind::Err(e) => return Err(e),
        };

        self.entries.insert(result.path, entry);
        Ok(())
    }

    pub(crate) fn finish_pending_io_events(&mut self) -> io::Result<()> {
        // SAFETY: we do not borrow self.joiners again
        let joiners = unsafe {
            std::mem::transmute::<&mut crate::state::Joiners, &mut crate::state::Joiners>(
                &mut self.joiners,
            )
        };

        let delete_joiner = &mut joiners.delete_joiner;

        joiners.runtime.block_on(async {
            while let Some(res) = delete_joiner.join_next().await {
                res?;
            }

            Ok(())
        })
    }
}
