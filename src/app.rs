pub struct App {
    controller: controller::Controller,
    typing: Option<String>,
}

impl App {
    pub fn run(&mut self, terminal: &mut tui::Tui) -> std::io::Result<()> {
        loop {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events()?;
        }
    }

    fn render_frame(&self, frame: &mut ratatui::Frame) {
        frame.render_widget(self, frame.size());
    }

    fn handle_events(&mut self) -> std::io::Result<()> {
        todo!()
    }
}

impl ratatui::widgets::Widget for &App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let title = Title::from(" N2P prototype ".bold());
        let block = Block::bordered()
            .title(title.alignment(Alignment::Center))
            .border_set(border::THICK);
        let items = ["Hello I am derp", "I am derpface", "I am groot"];
        let list = List::new(items).block(block);

        list.render(area, buf)
    }
}

use ratatui::style::Stylize as _;
use ratatui::widgets::Widget as _;

use ratatui::layout::Alignment;
use ratatui::symbols::border;
use ratatui::widgets::block::title::Title;
use ratatui::widgets::Block;
use ratatui::widgets::List;

use crate::controller;
use crate::tui;
