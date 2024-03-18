use std::{env, path::PathBuf};

use clap::{CommandFactory, Parser};
use tracing::error;

use runger::{
    cli::{self, Args},
    config::Config,
    state::State,
    terminal, Result,
};

fn run(path: PathBuf, config_path: PathBuf) -> Result<()> {
    let config = Config::new(config_path)?;

    let state = &mut State::new(path, config.required_columns)?;

    let mut terminal = terminal::init()?;

    while !state.handle_events()? {
        terminal.draw(state.ui())?;
    }

    terminal::close(terminal)?;
    Ok(())
}

fn main() {
    let args = cli::Args::parse();
    args.validate();

    let path = args.path.unwrap_or_else(|| match env::current_dir() {
        Ok(path) => path,
        Err(e) => Args::command()
            .error(
                clap::error::ErrorKind::InvalidValue,
                format!("unable to open given path: {e}"),
            )
            .exit(),
    });

    let config_path = args.config.unwrap_or_else(|| {
        let Some(mut config_path) = dirs::config_dir() else {
            Args::command()
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
        runger::init_logging(
            args.log_file.unwrap_or_else(|| {
                let Some(mut data_dir) = dirs::data_dir() else {
                    Args::command()
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
