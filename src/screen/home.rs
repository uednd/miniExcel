use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::Style,
    text::{Line, Span},
};

use crate::{
    model::{
        limits::{MAX_COLUMNS, MAX_ROWS},
        workbook::Workbook,
    },
    theme::Theme,
    widget::{
        logo::{LOGO_HEIGHT, Logo},
        tabs::{TabAction, Tabs},
    },
};

use super::{Screen, ScreenCommand};

pub struct MenuScreen {
    theme: Theme,
    cwd: String,
    logo: Logo,
    tabs: Tabs,
}

impl MenuScreen {
    pub fn new(theme: Theme, cwd: String) -> Self {
        Self {
            theme,
            cwd,
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
        match self.tabs.handle_key(key) {
            Some(TabAction::Handled) => Some(ScreenCommand::Stay),
            Some(TabAction::CreateTable(name)) => {
                let path = format!("{}/{}.mxlsx", self.cwd, name);
                let wb = Workbook::new(name, MAX_COLUMNS, MAX_ROWS);
                if wb.save(&path).is_ok() {
                    Some(ScreenCommand::OpenEditor { path })
                } else {
                    Some(ScreenCommand::Stay)
                }
            }
            None => Some(ScreenCommand::Stay),
        }
    }

    fn footer_hint(&self) -> Option<Line<'static>> {
        Some(Line::from(vec![
            Span::styled("● 提示", Style::default().fg(self.theme.accent)),
            Span::styled(" 使用 ", Style::default().fg(self.theme.text_dim)),
            Span::styled("Tab", Style::default().fg(self.theme.text)),
            Span::styled(" 切换标签", Style::default().fg(self.theme.text_dim)),
        ]))
    }
}
