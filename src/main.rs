use std::env;
use std::path::PathBuf;
use std::time::Duration;

use clap::{CommandFactory, Parser};
use handle_events::HandledEvent;
use indexmap::IndexMap;
use tracing::error;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

pub(crate) use crate::config::Config;
pub(crate) use crate::error::*;
pub(crate) use crate::state::entry::{Entry, EntryType};
pub(crate) use crate::state::{Command, DeleteCommand, State};

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
pub fn init_logging(mut log_file_path: PathBuf) -> Result<()> {
    if log_file_path.is_dir() {
        return Err(cli::Error::LogFilePathIsDirectory(log_file_path).into());
    }

    let log_file_name = match log_file_path.file_name().map(ToOwned::to_owned) {
        Some(path) if log_file_path.pop() => path,
        _ => return Err(cli::Error::InvalidLogFilePath(log_file_path).into()),
    };
    let log_dir_path = log_file_path;

    // log to file
    let logfile = tracing_appender::rolling::never(log_dir_path, log_file_name);

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .pretty()
                .with_span_events(tracing_subscriber::fmt::format::FmtSpan::ENTER)
                .with_writer(logfile)
                .with_filter(EnvFilter::from_default_env()),
        )
        .init();

    Ok(())
}

fn run(path: PathBuf, config_path: PathBuf) -> Result<()> {
    let config = match Config::new(config_path) {
        Ok(config) => config,
        Err(e) => {
            tracing::error!("unable to load config: {e}");
            Config::default()
        }
    };

    let state = &mut State::new(path, config)?;

    let mut terminal = terminal::init()?;

    loop {
        match state.handle_events() {
            Ok(HandledEvent::Exit) => break,
            Ok(HandledEvent::Redraw) => _ = terminal.draw(state.ui())?,
            Ok(HandledEvent::Nothing) => {}
            Err(e) => {
                tracing::error!("unable to handle events: {e}");

                // TODO: make show_error_duration configurable
                state.command_palette.set_error(e, Duration::from_secs(5));
            }
        }
    }

    state.finish_pending_io_events()?;

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
        )
        .expect("unable to init logging, exiting");
    }

    if let Err(e) = run(path, config_path) {
        error!("{e:?} : {e}");
    }
}
