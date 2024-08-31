#[derive(Default)]
pub struct Topics {
    list_state: ratatui::widgets::ListState,
}

impl components::Component for Topics {
    fn update(&mut self, event: crossterm::event::KeyEvent) -> components::Effect {
        components::Effect::Nothing
    }

    fn render(
        &mut self,
        model: &crate::model::Model,
        area: ratatui::layout::Rect,
        buf: &mut ratatui::buffer::Buffer,
    ) {
    }
}

use crate::components;
