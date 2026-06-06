pub mod home;

use crossterm::event::KeyEvent;
use ratatui::{Frame, layout::Rect, text::Line};

#[allow(dead_code)]
pub enum ScreenCommand {
    Stay,
    Navigate(Box<dyn Screen>),
}

pub trait Screen {
    fn render(&self, frame: &mut Frame, area: Rect);

    fn handle_key(&mut self, key: KeyEvent) -> Option<ScreenCommand>;

    fn footer_hint(&self) -> Option<Line<'static>> {
        None
    }
}
