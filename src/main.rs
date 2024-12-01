use std::env;
use std::path::PathBuf;
use std::time::Duration;

use clap::{CommandFactory, Parser};
use indexmap::IndexMap;
use tracing::error;

pub(crate) use crate::config::Config;
pub(crate) use crate::error::*;
pub(crate) use crate::state::entry::{Entry, EntryType};
#[expect(unused_imports)]
pub(crate) use crate::state::Command;
pub(crate) use crate::state::State;

pub mod cli;
pub mod config;
pub mod error;
pub mod handle_events;
pub mod state;
pub mod terminal;
pub mod ui;

type Map<K, V> = IndexMap<K, V, ahash::random_state::RandomState>;

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
        cli::LogLevel::Trace => tracing::Level::TRACE,
    };

    // log to file
    let logfile = tracing_appender::rolling::never(log_dir_path, log_file_name);

    #[cfg(not(debug_assertions))]
    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_writer(logfile)
        .init();

    #[cfg(debug_assertions)]
    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::ENTER)
        .with_writer(logfile)
        .init();

    Ok(())
}

fn run(path: PathBuf, config_path: PathBuf) -> Result<()> {
    let config = Config::new(config_path)?;

    let state = &mut State::new(path, config)?;

    let mut terminal = terminal::init()?;

    loop {
        let res = state.handle_events();
        match res {
            Ok(true) => break,
            Ok(false) => {}
            Err(e) => {
                // TODO: make show_error_duration configurable
                state.command_palette.set_error(e, Duration::from_secs(5));
            }
        }

        terminal.draw(state.ui())?;
    }

    terminal::close(terminal)?;
    Ok(())
}

fn main() {
    let args = crate::cli::Args::parse();
    args.validate();

    let path = args.path.unwrap_or_else(|| match env::current_dir() {
        Ok(path) => path,
        Err(e) => crate::cli::Args::command()
            .error(
                clap::error::ErrorKind::InvalidValue,
                format!("unable to open given path: {e}"),
            )
            .exit(),
    });

    let config_path = args.config.unwrap_or_else(|| {
        let Some(mut config_path) = dirs::config_dir() else {
            crate::cli::Args::command()
                .error(
                    clap::error::ErrorKind::InvalidValue,
                    "Unsupported OS: please set XDG_DATA_HOME env var or provide with config file path",
                )
                .exit()
        };
        config_path.push("runger");
        config_path.push("config.lua");
        config_path
    });

    if !args.quite {
        init_logging(
            args.log_file.unwrap_or_else(|| {
                let Some(mut data_dir) = dirs::data_dir() else {
                    crate::cli::Args::command()
                        .error(
                            clap::error::ErrorKind::InvalidValue,
                            "Unsupported OS: please set XDG_DATA_HOME env var or provide with log file path",
                        )
                        .exit()
                };
                data_dir.push("runger");
                data_dir.push("logs");
                data_dir
            }),
            args.log_level,
        )
        .expect("unable to init logging, exiting");
    }

    if let Err(e) = run(path, config_path) {
        error!("{e:?} : {e}");
    }
}
