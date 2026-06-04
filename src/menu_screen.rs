//! 主菜单画面。
//!
//! 包含 Logo、横向导航菜单及居中布局逻辑。

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    widgets::Paragraph,
};

use crate::{
    logo::{LOGO_HEIGHT, Logo},
    menu::{Menu, MenuAction},
    screen::{Screen, ScreenCommand},
};

/// logo 到菜单的间距。
const LOGO_MENU_GAP: u16 = 2;

/// 主菜单画面。
pub struct MenuScreen {
    logo: Logo,
    menu: Menu,
}

impl MenuScreen {
    pub fn new() -> Self {
        Self {
            logo: Logo::new(),
            menu: Menu::new(),
        }
    }
}

impl Screen for MenuScreen {
    fn render(&self, frame: &mut Frame, area: Rect, hint: Option<&str>) {
        let [_, logo_area, _, menu_area, hint_area, _] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(LOGO_HEIGHT),
            Constraint::Length(LOGO_MENU_GAP),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .areas(area);

        self.logo.render(frame, logo_area);
        self.menu.render(frame, menu_area);

        if let Some(text) = hint {
            frame.render_widget(Paragraph::new(text).centered().yellow().bold(), hint_area);
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> Option<ScreenCommand> {
        match key.code {
            KeyCode::Left => {
                self.menu.move_left();
                Some(ScreenCommand::Stay)
            }
            KeyCode::Right => {
                self.menu.move_right();
                Some(ScreenCommand::Stay)
            }
            KeyCode::Enter => match self.menu.selected_action() {
                MenuAction::NewTable => Some(ScreenCommand::Stay),
                MenuAction::Recent => Some(ScreenCommand::Stay),
                MenuAction::Settings => Some(ScreenCommand::Stay),
            },
            KeyCode::Char(_) => Some(ScreenCommand::Stay),
            _ => None,
        }
    }
}
