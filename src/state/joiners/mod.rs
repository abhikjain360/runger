use std::io;

pub(crate) use read_dir::*;

mod read_dir;

#[cfg_attr(debug_assertions, derive(Debug))]
pub(crate) struct Joiners {
    pub(crate) read_dir_joiners: ReadDirJoiner,
    pub(crate) runtime: tokio::runtime::Runtime,
}

impl Joiners {
    pub(super) fn new() -> io::Result<Self> {
        Ok(Self {
            read_dir_joiners: ReadDirJoiner::new(),
            runtime: tokio::runtime::Builder::new_current_thread()
                .enable_time()
                .build()?,
        })
    }

    #[expect(dead_code)]
    pub(crate) fn is_empty(&self) -> bool {
        self.read_dir_joiners.is_empty()
    }
}
