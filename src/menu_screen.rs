//! 主菜单画面。
//!
//! 包含 Logo、横向导航菜单、底栏，以及居中布局逻辑。

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Style, Stylize},
    widgets::{Block, Paragraph},
};

use crate::{
    footer::Footer,
    logo::{LOGO_HEIGHT, Logo},
    menu::{Menu, MenuAction},
    screen::{Screen, ScreenCommand},
};

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// logo 到菜单的间距。
const LOGO_MENU_GAP: u16 = 2;

/// 主菜单画面。
pub struct MenuScreen {
    logo: Logo,
    menu: Menu,
    footer: Footer,
}

impl MenuScreen {
    /// 创建主菜单画面实例。
    pub fn new() -> Self {
        let cwd_path = std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| String::from("."));
        let cwd = replace_homedir::replace_homedir(&cwd_path, "~");
        Self {
            logo: Logo::new(),
            menu: Menu::new(),
            footer: Footer::new(cwd, APP_VERSION.to_string()),
        }
    }
}

impl Screen for MenuScreen {
    /// 渲染主菜单画面。
    fn render(&self, frame: &mut Frame, hint: Option<&str>) {
        let area = frame.area();

        frame.render_widget(
            Block::new().style(Style::default().bg(Color::Rgb(10, 10, 10))),
            area,
        );

        let [body, footer_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(2)]).areas(area);

        let [_, logo_area, _, menu_area, hint_area, _] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(LOGO_HEIGHT),
            Constraint::Length(LOGO_MENU_GAP),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .areas(body);

        self.logo.render(frame, logo_area);
        self.menu.render(frame, menu_area);
        self.footer.render(frame, footer_area);

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
