use std::fs;
use std::io;

pub(crate) type ReadDirJoiner = tokio::task::JoinSet<io::Result<fs::ReadDir>>;

pub(crate) struct Joiners {
    pub(crate) read_dir_joiners: ReadDirJoiner,
    pub(crate) runtime: tokio::runtime::Runtime,
}

impl Joiners {
    pub(super) fn new() -> io::Result<Self> {
        Ok(Self {
            read_dir_joiners: ReadDirJoiner::new(),
            runtime: tokio::runtime::Builder::new_current_thread().build()?,
        })
    }

    #[expect(dead_code)]
    pub(crate) fn is_empty(&self) -> bool {
        self.read_dir_joiners.is_empty()
    }
}
