use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Paragraph},
};
use std::path::PathBuf;

use crate::screen::EventResult;
use crate::{
    model::recent::RecentFile,
    theme::Theme,
    widget::{
        input::Input,
        recent_list::{RecentList, RecentListCommand},
    },
};

#[derive(Debug, PartialEq, Eq)]
pub enum TabCommand {
    OpenTable(String),
    OpenRecent(PathBuf),
    RemoveRecent(PathBuf),
}

pub enum TabPage {
    Recent(&'static str, RecentList),
    OpenTable(&'static str, Input),
    Settings(&'static str),
}

pub struct Tabs {
    selected: usize,
    theme: Theme,
    pages: [TabPage; 3],
}

impl Tabs {
    pub fn new(theme: Theme, recent_files: Vec<RecentFile>) -> Self {
        Self {
            selected: 0,
            theme,
            pages: [
                TabPage::Recent("最近打开", RecentList::new(recent_files)),
                TabPage::OpenTable("打开表格", Input::new(theme)),
                TabPage::Settings("设置"),
            ],
        }
    }

    pub fn next(&mut self) {
        self.selected = (self.selected + 1) % self.pages.len();
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let centered: [Rect; 3] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Percentage(80),
            Constraint::Fill(1),
        ])
        .areas(area);

        let [tab_bar_area, tab_content_area] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(centered[1]);

        frame.render_widget(
            Block::default().style(Style::default().bg(self.theme.surface_alt)),
            tab_bar_area,
        );
        self.render_tab_bar(frame, tab_bar_area);

        frame.render_widget(
            Block::default().style(Style::default().bg(self.theme.surface)),
            tab_content_area,
        );
        self.pages[self.selected].render(frame, tab_content_area, self.theme);
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> EventResult<TabCommand> {
        match key.code {
            KeyCode::Tab => {
                self.next();
                EventResult::Handled
            }
            // 由当前 Tab content 处理
            _ => self.pages[self.selected].handle_key(key),
        }
    }

    pub fn footer_hint(&self) -> Line<'static> {
        match &self.pages[self.selected] {
            TabPage::Recent(_, recent) if recent.is_empty() => {
                hint_line(self.theme, &[("Tab", "切换标签")])
            }
            TabPage::Recent(_, _) => hint_line(
                self.theme,
                &[
                    ("Enter", "打开"),
                    ("Delete", "移除"),
                    ("↑ / ↓", "选择"),
                    ("Tab", "切换标签"),
                ],
            ),
            TabPage::OpenTable(_, _) => {
                hint_line(self.theme, &[("Enter", "打开或创建"), ("Tab", "切换标签")])
            }
            TabPage::Settings(_) => hint_line(self.theme, &[("Tab", "切换标签")]),
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
            TabPage::Recent(l, _) | TabPage::OpenTable(l, _) | TabPage::Settings(l) => l,
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect, theme: Theme) {
        match self {
            TabPage::Recent(_, recent) => {
                recent.render(frame, area, theme);
            }

            TabPage::OpenTable(_, input) => {
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
                    Paragraph::new("输入后按 Enter 打开或创建表格")
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

    fn handle_key(&mut self, key: KeyEvent) -> EventResult<TabCommand> {
        match self {
            TabPage::Recent(_, recent) => match recent.handle_key(key) {
                EventResult::Command(RecentListCommand::Open(path)) => {
                    EventResult::Command(TabCommand::OpenRecent(path))
                }
                EventResult::Command(RecentListCommand::Remove(path)) => {
                    EventResult::Command(TabCommand::RemoveRecent(path))
                }
                EventResult::Handled => EventResult::Handled,
                EventResult::Ignored => EventResult::Ignored,
            },
            TabPage::OpenTable(_, input) => match key.code {
                KeyCode::Enter => {
                    let name = table_name(input);
                    EventResult::Command(TabCommand::OpenTable(name))
                }
                KeyCode::Backspace => {
                    input.delete_char();
                    EventResult::Handled
                }
                KeyCode::Char(c) => {
                    input.insert_char(c);
                    EventResult::Handled
                }
                _ => EventResult::Ignored,
            },
            _ => EventResult::Ignored,
        }
    }
}

fn table_name(input: &Input) -> String {
    let name = input.buffer();
    if name.is_empty() {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        format!("表格_{}", ts)
    } else {
        name.to_string()
    }
}

fn hint_line(theme: Theme, pairs: &[(&'static str, &'static str)]) -> Line<'static> {
    let mut spans = vec![
        Span::styled("● 提示", Style::default().fg(theme.accent)),
        Span::styled(" ", Style::default().fg(theme.text_dim)),
    ];

    for (index, (key, label)) in pairs.iter().enumerate() {
        if index > 0 {
            spans.push(Span::styled("  ", Style::default().fg(theme.text_dim)));
        }
        spans.push(Span::styled(*key, Style::default().fg(theme.text)));
        spans.push(Span::styled(
            format!(" {}", label),
            Style::default().fg(theme.text_dim),
        ));
    }

    Line::from(spans)
}
