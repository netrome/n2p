pub struct App {
    controller: controller::Controller,
    components: Components,
    focus: Focus,
    key_pair: identity::Keypair,
    exit: bool,
}

impl App {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            controller: controller::Controller::new()?,
            components: Components::new(),
            focus: Focus::MessageInput,
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
            (crossterm::event::KeyModifiers::CONTROL, crossterm::event::KeyCode::Char('t')) => {
                self.focus = Focus::Topics;
            }
            (crossterm::event::KeyModifiers::CONTROL, crossterm::event::KeyCode::Char('y')) => {
                self.focus = Focus::MessageInput;
            }
            _event => {
                let effect = self.components.update(self.focus, key_event);
                match effect {
                    components::Effect::SendMessage(msg) => self.send_message(msg),
                    components::Effect::ViewTopic(topic) => self.components.chat_view.view(topic),
                    components::Effect::Return => self.focus = Focus::MessageInput,
                    _ => (),
                }
            }
        }
    }

    fn send_message(&mut self, msg: String) {
        let now = time::OffsetDateTime::now_utc();
        let created_at = time::PrimitiveDateTime::new(now.date(), now.time());
        let note = note::Note {
            topic: self
                .components
                .topics
                .selected_topic()
                .expect("No topic")
                .to_string(),
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
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Length(15), Constraint::Fill(1)])
            .split(area);

        let inner_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Fill(1), Constraint::Length(5)])
            .split(*layout.get(1).expect("impossibru"));

        self.components.topics.render(&self.controller.model(), *layout.first().expect("impossibru"), buf);

        self.components.chat_view.render(
            &self.controller.model(),
            *inner_layout.first().expect("impossibru"),
            buf,
        );

        self.components.message_input.render(
            &self.controller.model(),
            *inner_layout.get(1).expect("impossibru"),
            buf,
        );
    }
}

pub struct Components {
    topics: components::topics::Topics,
    chat_view: components::chat_view::ChatView,
    message_input: components::message_input::MessageInput,
}

impl Components {
    fn new() -> Self {
        let chat_view = components::chat_view::ChatView::new("Derp".to_string());
        let message_input = Default::default();
        let topics = components::topics::Topics::new();

        Self {
            chat_view,
            message_input,
            topics,
        }
    }

    fn update(&mut self, focus: Focus, event: crossterm::event::KeyEvent) -> components::Effect {
        match focus {
            Focus::Topics => self.topics.update(event),
            Focus::ChatView => self.chat_view.update(event),
            Focus::MessageInput => self.message_input.update(event),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Focus {
    Topics,
    ChatView,
    MessageInput,
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
