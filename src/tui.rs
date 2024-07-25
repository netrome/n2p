pub type Tui = ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>;

pub fn init_terminal() -> std::io::Result<Tui> {
    crossterm::execute!(std::io::stdout(), crossterm::terminal::EnterAlternateScreen)?;
    crossterm::terminal::enable_raw_mode()?;
    ratatui::Terminal::new(ratatui::backend::CrosstermBackend::new(std::io::stdout()))
}

pub fn restore_terminal() -> std::io::Result<()> {
    crossterm::execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen)?;
    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}
