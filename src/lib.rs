use std::path::PathBuf;

use indexmap::IndexMap;

pub use crate::error::*;

pub mod cli;
pub mod config;
pub mod error;
pub mod handle_events;
pub mod state;
pub mod terminal;
pub mod ui;

type RungerMap<K, V> = IndexMap<K, V, ahash::random_state::RandomState>;

/// Initialises logging. The returned guard shouldn't be dropped otherwise there is guarantee that
/// all logs will be flushed.
pub fn init_logging(mut log_file_path: PathBuf, log_level: cli::LogLevel) -> Result<()> {
    if log_file_path.is_dir() {
        return Err(cli::Error::LogFilePathIsDirectory(log_file_path).into());
    }

    let log_file_name = match log_file_path.file_name().map(ToOwned::to_owned) {
        Some(path) if log_file_path.pop() => path,
        _ => return Err(cli::Error::InvalidLogFilePath(log_file_path).into()),
    };
    let log_dir_path = log_file_path;

    let log_level = match log_level {
        cli::LogLevel::Error => tracing::Level::ERROR,
        cli::LogLevel::Warn => tracing::Level::WARN,
        cli::LogLevel::Info => tracing::Level::INFO,
        cli::LogLevel::Debug => tracing::Level::DEBUG,
    };

    // log to file
    let logfile = tracing_appender::rolling::never(log_dir_path, log_file_name);

    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_writer(logfile)
        .init();

    Ok(())
}
