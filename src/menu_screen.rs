use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    widgets::{Block, Paragraph},
};

use crate::{
    input::Input,
    logo::{Logo, LOGO_HEIGHT},
    screen::{Screen, ScreenCommand},
    tabs::Tabs,
    theme::Theme,
};

pub struct MenuScreen {
    theme: Theme,
    logo: Logo,
    tabs: Tabs,
    input: Input,
}

impl MenuScreen {
    pub fn new(theme: Theme) -> Self {
        Self {
            theme,
            logo: Logo::new(theme),
            tabs: Tabs::new(theme),
            input: Input::new(theme),
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

        let centered: [Rect; 3] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Percentage(80),
            Constraint::Fill(1),
        ])
        .areas(body_area);

        let [tab_bar_area, tab_content_area] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(centered[1]);

        frame.render_widget(
            Block::default().style(Style::default().bg(self.theme.surface_alt)),
            tab_bar_area,
        );
        self.tabs.render(frame, tab_bar_area);

        frame.render_widget(
            Block::default().style(Style::default().bg(self.theme.surface)),
            tab_content_area,
        );

        match self.tabs.selected() {
            0 => {
                frame.render_widget(
                    Paragraph::new("暂无最近打开的文件")
                        .centered()
                        .fg(self.theme.text_dim),
                    tab_content_area,
                );
            }
            1 => {
                let [_, input_area, _, hint_area, _] = Layout::vertical([
                    Constraint::Fill(1),
                    Constraint::Length(3),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Fill(1),
                ])
                .areas(tab_content_area);

                let input_centered: [Rect; 3] = Layout::horizontal([
                    Constraint::Fill(1),
                    Constraint::Length(40),
                    Constraint::Fill(1),
                ])
                .areas(input_area);

                self.input.render(frame, input_centered[1]);

                frame.render_widget(
                    Paragraph::new("输入后按 Enter 创建表格")
                        .centered()
                        .fg(self.theme.text_dim),
                    hint_area,
                );
            }
            2 => {
                frame.render_widget(
                    Paragraph::new("设置功能即将上线")
                        .centered()
                        .fg(self.theme.text_dim),
                    tab_content_area,
                );
            }
            _ => {}
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> Option<ScreenCommand> {
        match key.code {
            KeyCode::Left => {
                self.tabs.prev();
                Some(ScreenCommand::Stay)
            }
            KeyCode::Right => {
                self.tabs.next();
                Some(ScreenCommand::Stay)
            }
            KeyCode::Enter => Some(ScreenCommand::Stay),
            KeyCode::Backspace if self.tabs.selected() == 1 => {
                self.input.delete_char();
                Some(ScreenCommand::Stay)
            }
            KeyCode::Char(c) if self.tabs.selected() == 1 => {
                self.input.insert_char(c);
                Some(ScreenCommand::Stay)
            }
            _ => Some(ScreenCommand::Stay),
        }
    }
}
