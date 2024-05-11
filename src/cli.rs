use clap::CommandFactory;
use std::path::PathBuf;

#[derive(Debug, Clone, clap::Parser)]
#[command(version)]
pub struct Args {
    // The level of logs to log.
    #[arg(short = 'l', long, default_value_t = LogLevel::Info, value_enum)]
    pub log_level: LogLevel,

    // Location of log file.
    #[arg(long)]
    pub log_file: Option<PathBuf>,

    // Whether to log logs or not.
    #[arg(short = 'q', long, default_value_t = false)]
    pub quite: bool,

    // Location of config file, otherwise uses system default.
    #[arg(short = 'c', long)]
    pub config: Option<PathBuf>,

    // Path from where to launch.
    pub path: Option<PathBuf>,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, strum::EnumIter, clap::ValueEnum,
)]
pub enum LogLevel {
    Error,
    Warn,
    #[default]
    Info,
    Debug,
    Trace,
}

impl Args {
    pub fn validate(&self) {
        if let Some(path) = &self.path {
            if !path.exists() || !path.is_dir() {
                Self::command()
                    .error(
                        clap::error::ErrorKind::InvalidValue,
                        "path given for logs is not a directory",
                    )
                    .exit();
            }
        }

        if let Some(path) = &self.config {
            if path.exists() && !path.is_file() {
                Self::command()
                    .error(
                        clap::error::ErrorKind::InvalidValue,
                        "path given for logs is not a file",
                    )
                    .exit();
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("invalid log level \"{0}\", should be debug, info, warn or error")]
    InvalidLogLevel(String),
    #[error("given path \"{0}\" for log file is a directory")]
    LogFilePathIsDirectory(PathBuf),
    #[error("given path \"{0}\" for log file is a invalid")]
    InvalidLogFilePath(PathBuf),
    #[error("given path \"{0}\" is a invalid")]
    InvalidFilePath(PathBuf),
}
