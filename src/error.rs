pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO Error {0:?}: {0}")]
    Io(#[from] std::io::Error),
    #[error("CLI: {0}")]
    Cli(#[from] crate::cli::Error),
    #[error("Config: Lua: {0}")]
    Lua(#[from] mlua::Error),
    #[error("Config: {0}")]
    Config(#[from] crate::config::Error),
}
