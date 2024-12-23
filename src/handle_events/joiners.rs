use std::io;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use futures::stream::FuturesUnordered;
use futures::{FutureExt, StreamExt};

use crate::state::ReadDirResult;

use super::HandledEvent;

impl crate::State {
    /// Returns `true` if timeout did not occur, that is, some IO event was handled. We should
    /// redraw.
    pub(super) fn poll_io_event(&mut self, timeout: Duration) -> io::Result<HandledEvent> {
        enum PollResult {
            Delete(io::Result<Arc<PathBuf>>),
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
            return Ok(HandledEvent::Nothing);
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
            return Ok(HandledEvent::Nothing);
        };

        match res {
            PollResult::Delete(res) => {
                let path = res?;
                self.delete_path_entry(path);
            }
            // TODO: verify that we do need to redraw, as we might have updated optimistically
            PollResult::ReadDir(res) => {
                self.handle_read_dir_event(res)?;
            }
            PollResult::Timeout => return Ok(HandledEvent::Nothing),
        }

        self.try_open_selected_path();
        Ok(HandledEvent::Redraw)
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
                let select_on_open =
                    self.entries
                        .swap_remove(&result.path)
                        .and_then(|entry| match entry.ty {
                            crate::EntryType::Waiting(unopened)
                            | crate::EntryType::Unopened(unopened) => unopened.select_on_open,
                            _ => None,
                        });

                crate::Entry::opened(
                    result.path.clone(),
                    entries,
                    self.config.clone(),
                    select_on_open,
                )
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
