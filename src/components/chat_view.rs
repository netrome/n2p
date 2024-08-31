pub struct ChatView {
    topic: String,
    list_state: ratatui::widgets::ListState,
}

impl ChatView {
    pub fn new(topic: String) -> Self {
        Self {
            topic,
            list_state: Default::default(),
        }
    }
}

impl components::Component for ChatView {
    fn update(&mut self, _event: crossterm::event::KeyEvent) -> components::Effect {
        components::Effect::Nothing
    }

    fn render(
        &mut self,
        model: &crate::model::Model,
        area: ratatui::layout::Rect,
        buf: &mut ratatui::buffer::Buffer,
    ) {
        let block = ratatui::widgets::Block::bordered().border_set(ratatui::symbols::border::THICK);

        let items: Vec<_> = model
            .topics
            .get(&self.topic)
            .cloned()
            .unwrap_or_default()
            .notes
            .values()
            .map(|note| note.inner.msg.clone())
            .collect();

        let list = ratatui::widgets::List::new(items)
            .block(block)
            .style(ratatui::style::Style::default())
            .fg(ratatui::style::Color::White)
            .highlight_style(
                ratatui::style::Style::default().add_modifier(ratatui::style::Modifier::ITALIC),
            );

        self.list_state.select_last();

        ratatui::widgets::StatefulWidget::render(list, area, buf, &mut self.list_state);
    }
}

use ratatui::style::Stylize as _;

use crate::components;
