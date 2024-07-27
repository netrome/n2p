#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut terminal = n2p::tui::init_terminal()?;

    let mut app = n2p::app::App::new();

    app.run(&mut terminal).await?;

    n2p::tui::restore_terminal()?;

    Ok(())
}
