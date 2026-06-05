use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style, Stylize},
    text::Line,
    widgets::{Block, Paragraph},
};

use crate::{theme::Theme, widget::input::Input};

pub enum TabPage {
    Recent(&'static str),
    NewTable(&'static str, Input),
    Settings(&'static str),
}

pub struct Tabs {
    selected: usize,
    theme: Theme,
    pages: [TabPage; 3],
}

impl Tabs {
    pub fn new(theme: Theme) -> Self {
        Self {
            selected: 0,
            theme,
            pages: [
                TabPage::Recent("最近打开"),
                TabPage::NewTable("新建表格", Input::new(theme)),
                TabPage::Settings("设置"),
            ],
        }
    }

    pub fn prev(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn next(&mut self) {
        if self.selected + 1 < self.pages.len() {
            self.selected += 1;
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let centered: [Rect; 3] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Percentage(80),
            Constraint::Fill(1),
        ])
        .areas(area);

        // Tab bar
        let [tab_bar_area, tab_content_area] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(centered[1]);

        frame.render_widget(
            Block::default().style(Style::default().bg(self.theme.surface_alt)),
            tab_bar_area,
        );
        self.render_tab_bar(frame, tab_bar_area);

        // Tab content
        frame.render_widget(
            Block::default().style(Style::default().bg(self.theme.surface)),
            tab_content_area,
        );
        self.pages[self.selected].render(frame, tab_content_area, self.theme);
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Left => {
                self.prev();
                true
            }
            KeyCode::Right => {
                self.next();
                true
            }
            // 由当前 Tab content 处理
            _ => self.pages[self.selected].handle_key(key),
        }
    }

    fn render_tab_bar(&self, frame: &mut Frame, area: Rect) {
        let chunks: [Rect; 3] = Layout::horizontal([Constraint::Fill(1); 3]).areas(area);

        let selected_style = Style::default()
            .fg(self.theme.accent_text)
            .bg(self.theme.accent)
            .add_modifier(Modifier::BOLD);
        let unselected_style = Style::default()
            .fg(self.theme.text)
            .bg(self.theme.surface_alt);

        for (i, page) in self.pages.iter().enumerate() {
            let style = if i == self.selected {
                selected_style
            } else {
                unselected_style
            };
            frame.render_widget(
                Paragraph::new(Line::from(page.label()))
                    .centered()
                    .style(style),
                chunks[i],
            );
        }
    }
}

impl TabPage {
    fn label(&self) -> &'static str {
        match self {
            TabPage::Recent(l) | TabPage::NewTable(l, _) | TabPage::Settings(l) => l,
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect, theme: Theme) {
        match self {
            TabPage::Recent(_) => {
                frame.render_widget(
                    Paragraph::new("暂无最近打开的文件")
                        .centered()
                        .fg(theme.text_dim),
                    area,
                );
            }

            TabPage::NewTable(_, input) => {
                let [_, input_area, _, hint_area, _] = Layout::vertical([
                    Constraint::Fill(1),
                    Constraint::Length(3),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Fill(1),
                ])
                .areas(area);

                let input_centered: [Rect; 3] = Layout::horizontal([
                    Constraint::Fill(1),
                    Constraint::Length(40),
                    Constraint::Fill(1),
                ])
                .areas(input_area);

                input.render(frame, input_centered[1]);

                frame.render_widget(
                    Paragraph::new("输入后按 Enter 创建表格")
                        .centered()
                        .fg(theme.text_dim),
                    hint_area,
                );
            }

            TabPage::Settings(_) => {
                frame.render_widget(
                    Paragraph::new("设置功能即将上线")
                        .centered()
                        .fg(theme.text_dim),
                    area,
                );
            }
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        match self {
            TabPage::NewTable(_, input) => match key.code {
                KeyCode::Backspace => {
                    input.delete_char();
                    true
                }
                // 用户的输入字符
                KeyCode::Char(c) => {
                    input.insert_char(c);
                    true
                }
                _ => false,
            },
            _ => false,
        }
    }
}
