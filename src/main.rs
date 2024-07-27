#[tokio::main]
async fn main() {
    let mut terminal = n2p::tui::init_terminal().expect("failed to init terminal");

    let mut app = n2p::app::App::new();

    app.run(&mut terminal).await.expect("failed to run app");

    n2p::tui::restore_terminal().expect("failed to restore terminal");
    println!("Hello, world!");
}
