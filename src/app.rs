pub struct App {
    controller: controller::Controller,
    typing: Option<String>,
    key_pair: identity::Keypair,
    exit: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            controller: controller::Controller::new(),
            typing: None,
            key_pair: identity::Keypair::generate_ed25519(),
            exit: false,
        }
    }
    pub async fn run(&mut self, terminal: &mut tui::Tui) -> anyhow::Result<()> {
        let mut event_stream = crossterm::event::EventStream::new();
        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            tokio::select! {
                _ = self.controller.poll() => {}

                event = event_stream.next() => {
                    if let Some(Ok(event)) = event {
                    self.handle_event(event);
                    } else {
                        panic!("oh nooo, what happened with the event stream?")
                    }
                }
            };
        }

        Ok(())
    }

    fn render_frame(&self, frame: &mut ratatui::Frame) {
        frame.render_widget(self, frame.size());
    }

    fn handle_event(&mut self, event: crossterm::event::Event) {
        match event {
            crossterm::event::Event::Key(key_event) => self.handle_key_event(key_event),
            other => {
                //println!("received other event: {other:?}");
            }
        }
    }

    fn handle_key_event(&mut self, key_event: crossterm::event::KeyEvent) {
        match (key_event.modifiers, key_event.code) {
            (crossterm::event::KeyModifiers::CONTROL, crossterm::event::KeyCode::Char('q')) => {
                self.exit = true;
            }
            (_, crossterm::event::KeyCode::Enter) => {
                self.on_enter();
            }
            (_, crossterm::event::KeyCode::Char(c)) => {
                self.type_char(c);
            }
            _ => {
                todo!()
            }
        }
    }

    fn on_enter(&mut self) {
        match &self.typing {
            Some(msg) => {
                self.send_message(msg.to_owned());
                self.typing = None;
            }
            None => {
                self.typing = Some(String::new());
            }
        }
    }

    fn type_char(&mut self, c: char) {
        let Some(msg) = self.typing.as_mut() else {
            return;
        };

        msg.push(c);
    }

    fn send_message(&mut self, msg: String) {
        let now = time::OffsetDateTime::now_utc();
        let created_at = time::PrimitiveDateTime::new(now.date(), now.time());
        let note = note::Note {
            topic: "Derp".to_owned(),
            msg,
            created_at,
        };

        let signed = note.sign(&self.key_pair).expect("failed to sign note");
        self.controller.send_note(signed);
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
        let items: Vec<_> = self
            .controller
            .model()
            .topics
            .get("Derp")
            .cloned()
            .unwrap_or_default()
            .notes
            .values()
            .map(|note| note.inner.msg.clone()).collect();
        //let items = ["Hello I am derp", "I am derpface", "I am groot"];
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

use libp2p::identity;

use crate::controller;
use crate::note;
use crate::note::Sign;
use crate::tui;
