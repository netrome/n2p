fn main() {
    n2p::tui::init_terminal().expect("failed to init terminal");

    n2p::tui::restore_terminal().expect("failed to restore terminal");
    println!("Hello, world!");
}
