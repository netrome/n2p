pub struct App {
    controller: controller::Controller,
    components: Components,
    key_pair: identity::Keypair,
    exit: bool,
}

impl App {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            controller: controller::Controller::new()?,
            components: Components::new(),
            key_pair: identity::Keypair::generate_ed25519(),
            exit: false,
        })
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

    fn render_frame(&mut self, frame: &mut ratatui::Frame) {
        frame.render_widget(self, frame.size());
    }

    fn handle_event(&mut self, event: crossterm::event::Event) {
        match event {
            crossterm::event::Event::Key(key_event) => self.handle_key_event(key_event),
            _other => {}
        }
    }

    fn handle_key_event(&mut self, key_event: crossterm::event::KeyEvent) {
        match (key_event.modifiers, key_event.code) {
            (crossterm::event::KeyModifiers::CONTROL, crossterm::event::KeyCode::Char('q')) => {
                self.exit = true;
            }
            _event => {
                let effect = self.components.message_input.update(key_event);
                match effect {
                    components::Effect::SendMessage(msg) => self.send_message(msg),
                    _ => (),
                }
            }
        }
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

impl ratatui::widgets::Widget for &mut App {
    fn render(self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer)
    where
        Self: Sized,
    {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        self.components.chat_view.render(
            &self.controller.model(),
            *layout.first().expect("impossibru"),
            buf,
        );

        self.components.message_input.render(
            &self.controller.model(),
            *layout.get(1).expect("impossibru"),
            buf,
        );
    }
}

pub struct Components {
    chat_view: components::chat_view::ChatView,
    message_input: components::message_input::MessageInput,
}

impl Components {
    fn new() -> Self {
        let chat_view = components::chat_view::ChatView::new("Derp".to_string());
        let message_input = Default::default();

        Self {
            chat_view,
            message_input,
        }
    }
}

use crate::components::Component as _;
use futures::StreamExt as _;

use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Layout;

use libp2p::identity;

use crate::components;
use crate::controller;
use crate::note;
use crate::note::Sign;
use crate::tui;
