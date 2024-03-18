use std::io;

use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::backend::CrosstermBackend;

pub type Terminal = ratatui::Terminal<CrosstermBackend<io::Stdout>>;

pub fn init() -> io::Result<Terminal> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    Terminal::new(CrosstermBackend::new(stdout))
}

pub fn close(mut terminal: Terminal) -> io::Result<()> {
    terminal.backend_mut().execute(LeaveAlternateScreen)?;
    disable_raw_mode()
}
