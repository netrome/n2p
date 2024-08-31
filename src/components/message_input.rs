#[derive(Default)]
pub struct MessageInput {
    text_area: tui_textarea::TextArea<'static>,
}

impl MessageInput {
    pub fn new() -> Self {
        Self::default()
    }

    fn get_message(&mut self) -> String {
        self.text_area.select_all();
        self.text_area.cut();
        self.text_area.yank_text()
    }
}

impl components::Component for MessageInput {
    fn update(&mut self, event: crossterm::event::KeyEvent) -> components::Effect {
        match (event.modifiers, event.code) {
            (crossterm::event::KeyModifiers::CONTROL, crossterm::event::KeyCode::Char('s')) => {
                components::Effect::SendMessage(self.get_message())
            }
            _ => {
                self.text_area.input(event);
                components::Effect::Nothing
            }
        }
    }

    fn render(
        &mut self,
        _model: &crate::model::Model,
        area: ratatui::layout::Rect,
        buf: &mut ratatui::buffer::Buffer,
    ) {
        self.text_area.widget().render(area, buf)
    }
}

use ratatui::widgets::Widget;

use crate::components;
