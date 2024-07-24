pub type Tui = ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>;

pub fn init_terminal() -> std::io::Result<Tui> {
    ratatui::crossterm::execute!(
        std::io::stdout(),
        ratatui::crossterm::terminal::EnterAlternateScreen
    )?;
    ratatui::crossterm::terminal::enable_raw_mode()?;
    ratatui::Terminal::new(ratatui::backend::CrosstermBackend::new(std::io::stdout()))
}

pub fn restore_terminal() -> std::io::Result<()> {
    ratatui::crossterm::execute!(
        std::io::stdout(),
        ratatui::crossterm::terminal::LeaveAlternateScreen
    )?;
    ratatui::crossterm::terminal::disable_raw_mode()?;
    Ok(())
}
