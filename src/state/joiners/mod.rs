use std::io;

use delete::*;
pub(crate) use read_dir::*;

mod delete;
mod read_dir;

pub(crate) struct Joiners {
    pub(crate) read_dir_joiner: ReadDirJoiner,
    pub(crate) delete_joiner: DeleteJoiner,
    pub(crate) runtime: tokio::runtime::Runtime,
}

impl Joiners {
    pub(super) fn new() -> io::Result<Self> {
        Ok(Self {
            read_dir_joiner: ReadDirJoiner::new(),
            delete_joiner: DeleteJoiner::new(),
            runtime: tokio::runtime::Builder::new_current_thread()
                .enable_time()
                .build()?,
        })
    }

    #[expect(dead_code)]
    pub(crate) fn is_empty(&self) -> bool {
        self.read_dir_joiner.is_empty() && self.delete_joiner.is_empty()
    }
}
