pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error("IO Error {0:?}: {0}")]
    Io(#[from] std::io::Error),
    #[error("CLI Error: {0}")]
    Cli(#[from] crate::cli::Error),
    #[error("Lua Error: {0}")]
    Lua(#[from] mlua::Error),
    #[error("Config Error: {0}")]
    Config(#[from] crate::config::Error),
    #[error("Command Error: {0}")]
    Command(#[from] crate::state::CommandError),
    #[cfg(debug_assertions)]
    #[error("random error")]
    Random,
}
