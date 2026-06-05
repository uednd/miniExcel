use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
};

use crate::{
    theme::Theme,
    widget::{
        logo::{LOGO_HEIGHT, Logo},
        tabs::Tabs,
    },
};

use super::{Screen, ScreenCommand};

pub struct MenuScreen {
    logo: Logo,
    tabs: Tabs,
}

impl MenuScreen {
    pub fn new(theme: Theme) -> Self {
        Self {
            logo: Logo::new(theme),
            tabs: Tabs::new(theme),
        }
    }
}

impl Screen for MenuScreen {
    fn render(&self, frame: &mut Frame, area: Rect) {
        let [_, logo_area, _, body_area, _] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(LOGO_HEIGHT),
            Constraint::Length(3),
            Constraint::Fill(3),
            Constraint::Fill(1),
        ])
        .areas(area);

        self.logo.render(frame, logo_area);
        self.tabs.render(frame, body_area);
    }

    fn handle_key(&mut self, key: KeyEvent) -> Option<ScreenCommand> {
        match key.code {
            KeyCode::Enter => Some(ScreenCommand::Stay),
            _ if self.tabs.handle_key(key) => Some(ScreenCommand::Stay),
            _ => Some(ScreenCommand::Stay),
        }
    }
}
