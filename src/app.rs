pub struct App {
    controller: controller::Controller,
    typing: Option<String>,
}

impl App {
    pub async fn run(&mut self, terminal: &mut tui::Tui) -> std::io::Result<()> {
        let mut event_stream = crossterm::event::EventStream::new();
        loop {
            terminal.draw(|frame| self.render_frame(frame))?;
            tokio::select! {
                _ = self.controller.poll() => {
                    todo!();
                }

                event = event_stream.next() => {
                    if let Some(Ok(event)) = event {
                    self.handle_event(event);
                    } else {
                        panic!("oh nooo, what happened with the event stream?")
                    }
                }
            };
        }
    }

    fn render_frame(&self, frame: &mut ratatui::Frame) {
        frame.render_widget(self, frame.size());
    }

    fn handle_event(&mut self, event: crossterm::event::Event) {
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

use futures::StreamExt as _;
use ratatui::style::Stylize as _;
use ratatui::widgets::Widget as _;

use ratatui::layout::Alignment;
use ratatui::symbols::border;
use ratatui::widgets::block::title::Title;
use ratatui::widgets::Block;
use ratatui::widgets::List;

use crate::controller;
use crate::tui;
