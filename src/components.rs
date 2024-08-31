pub trait Component {
    fn update(&mut self, event: crossterm::event::KeyEvent) -> Effect;
    fn render(
        &mut self,
        model: &model::Model,
        area: ratatui::layout::Rect,
        buf: &mut ratatui::buffer::Buffer,
    );
}

pub enum Effect {
    SendMessage(String),
    Nothing,
}

pub mod chat_view;
pub mod message_input;
pub mod topics;

use crate::model;
