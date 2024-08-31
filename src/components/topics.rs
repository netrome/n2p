pub struct Topics {
    known_topics: Vec<String>,
    list_state: ratatui::widgets::ListState,
}

impl Topics {
    pub fn new() -> Self {
        Self {
            known_topics: vec!["Derp".to_string(), "Flerp".to_string(), "Herp".to_string()],
            list_state: Default::default(),
        }
    }

    pub fn selected_topic(&self) -> Option<&str> {
        self.list_state
            .selected()
            .and_then(|selected| self.known_topics.get(selected).map(|s| s.as_str()))
    }
}

impl components::Component for Topics {
    fn update(&mut self, event: crossterm::event::KeyEvent) -> components::Effect {
        match event.code {
            crossterm::event::KeyCode::Up => self.list_state.select_previous(),
            crossterm::event::KeyCode::Down => self.list_state.select_next(),
            crossterm::event::KeyCode::Enter => return components::Effect::Return,
            _ => (),
        };

        if let Some(topic) = self.selected_topic() {
            components::Effect::ViewTopic(topic.to_string())
        } else {
            components::Effect::Nothing
        }
    }

    fn render(
        &mut self,
        _model: &model::Model, // TODO: Reconcile with model
        area: ratatui::layout::Rect,
        buf: &mut ratatui::buffer::Buffer,
    ) {
        let block = ratatui::widgets::Block::bordered().border_set(ratatui::symbols::border::THICK);

        let items: Vec<_> = self.known_topics.clone();

        let list = ratatui::widgets::List::new(items)
            .block(block)
            .style(ratatui::style::Style::default())
            .fg(ratatui::style::Color::White)
            .highlight_style(
                ratatui::style::Style::default()
                    .bold()
                    .bg(ratatui::style::Color::Gray),
            );

        ratatui::widgets::StatefulWidget::render(list, area, buf, &mut self.list_state)
    }
}

use ratatui::style::Stylize;

use crate::components;
use crate::model;
