pub mod editor;
pub mod home;

use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{Frame, layout::Rect, text::Line};

pub enum ScreenCommand {
    Stay,
    OpenEditor { path: String },
    GoHome,
}

pub trait Screen {
    fn render(&self, frame: &mut Frame, area: Rect);

    fn handle_key(&mut self, key: KeyEvent) -> Option<ScreenCommand>;

    fn handle_scroll(&mut self, _event: MouseEvent) -> Option<ScreenCommand> {
        None
    }

    fn footer_hint(&self) -> Option<Line<'static>> {
        None
    }

    fn footer_status(&self) -> Option<Line<'static>> {
        None
    }
}
