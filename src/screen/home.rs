use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    text::Line,
};
use std::path::PathBuf;

use crate::{
    model::{recent::RecentFile, table_path::resolve_table_path},
    theme::Theme,
    widget::{
        logo::{LOGO_HEIGHT, Logo},
        tabs::{TabCommand, Tabs},
    },
};

use super::{EventResult, Screen, ScreenCommand};

pub struct MenuScreen {
    cwd: PathBuf,
    logo: Logo,
    tabs: Tabs,
}

impl MenuScreen {
    pub fn new(theme: Theme, cwd: PathBuf, recent_files: Vec<RecentFile>) -> Self {
        Self {
            cwd,
            logo: Logo::new(theme),
            tabs: Tabs::new(theme, recent_files),
        }
    }
}

impl Screen for MenuScreen {
    fn render(&mut self, frame: &mut Frame, area: Rect) {
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

    fn handle_key(&mut self, key: KeyEvent) -> EventResult<ScreenCommand> {
        match self.tabs.handle_key(key) {
            EventResult::Handled => EventResult::Handled,
            EventResult::Command(TabCommand::OpenTable(input)) => {
                let path = resolve_table_path(&input, &self.cwd);
                EventResult::Command(ScreenCommand::OpenEditor { path })
            }
            EventResult::Command(TabCommand::OpenRecent(path)) => {
                EventResult::Command(ScreenCommand::OpenEditor { path })
            }
            EventResult::Command(TabCommand::RemoveRecent(path)) => {
                EventResult::Command(ScreenCommand::RemoveRecent { path })
            }
            EventResult::Ignored => EventResult::Handled,
        }
    }

    fn footer_hint(&self) -> Option<Line<'static>> {
        Some(self.tabs.footer_hint())
    }
}
