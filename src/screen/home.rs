use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    text::Line,
};
use std::path::PathBuf;

use crate::{
    model::{document::resolve_table_path, recent::RecentFile},
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
    status_message: Option<String>,
}

impl MenuScreen {
    pub fn new(theme: Theme, cwd: PathBuf, recent_files: Vec<RecentFile>) -> Self {
        Self::with_status(theme, cwd, recent_files, None)
    }

    pub fn with_status(
        theme: Theme,
        cwd: PathBuf,
        recent_files: Vec<RecentFile>,
        status_message: Option<String>,
    ) -> Self {
        Self {
            cwd,
            logo: Logo::new(theme),
            tabs: Tabs::new(theme, recent_files),
            status_message,
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

    fn footer_status(&self) -> Option<Line<'static>> {
        self.status_message.as_ref().map(|message| {
            Line::styled(
                message.clone(),
                ratatui::style::Style::default().fg(ratatui::style::Color::Red),
            )
        })
    }
}
