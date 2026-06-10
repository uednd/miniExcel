use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Paragraph},
};

use crate::{
    model::{document::resolve_table_path, recent::RecentFile},
    screen::EventResult,
    theme::Theme,
    widget::{input::Input, recent_list::RecentList},
};

#[derive(Debug, PartialEq, Eq)]
pub enum HomeAction {
    Open(PathBuf),
    RemoveRecent(PathBuf),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum HomeTab {
    Recent,
    OpenTable,
    Settings,
}

const TABS: [HomeTab; 3] = [HomeTab::Recent, HomeTab::OpenTable, HomeTab::Settings];

/// 首页交互流程。
///
/// 该模块集中管理首页 Tab、路径输入、最近文件选择和 footer 提示。
pub struct HomeFlow {
    selected_tab: usize,
    open_input: Input,
    recent_list: RecentList,
    theme: Theme,
    cwd: PathBuf,
}

impl HomeFlow {
    pub fn new(theme: Theme, cwd: PathBuf, recent_files: Vec<RecentFile>) -> Self {
        Self {
            selected_tab: 0,
            open_input: Input::new(theme),
            recent_list: RecentList::new(recent_files),
            theme,
            cwd,
        }
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
        self.render_current_tab(frame, tab_content_area);
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> EventResult<HomeAction> {
        if key.code == KeyCode::Tab {
            self.next_tab();
            return EventResult::Handled;
        }

        match self.current_tab() {
            HomeTab::Recent => self.handle_recent_key(key),
            HomeTab::OpenTable => self.handle_open_table_key(key),
            HomeTab::Settings => EventResult::Ignored,
        }
    }

    pub fn footer_hint(&self) -> Line<'static> {
        match self.current_tab() {
            HomeTab::Recent if self.recent_list.is_empty() => {
                hint_line(self.theme, &[("Tab", "切换标签")])
            }
            HomeTab::Recent => hint_line(
                self.theme,
                &[
                    ("Enter", "打开"),
                    ("Delete", "移除"),
                    ("↑ / ↓", "选择"),
                    ("Tab", "切换标签"),
                ],
            ),
            HomeTab::OpenTable => {
                hint_line(self.theme, &[("Enter", "打开或创建"), ("Tab", "切换标签")])
            }
            HomeTab::Settings => hint_line(self.theme, &[("Tab", "切换标签")]),
        }
    }

    fn next_tab(&mut self) {
        self.selected_tab = (self.selected_tab + 1) % TABS.len();
    }

    fn current_tab(&self) -> HomeTab {
        TABS[self.selected_tab]
    }

    fn handle_recent_key(&mut self, key: KeyEvent) -> EventResult<HomeAction> {
        if self.recent_list.is_empty() {
            return EventResult::Ignored;
        }

        match key.code {
            KeyCode::Up => {
                self.recent_list.move_up();
                EventResult::Handled
            }
            KeyCode::Down => {
                self.recent_list.move_down();
                EventResult::Handled
            }
            KeyCode::Enter => {
                let Some(path) = self.recent_list.selected_path() else {
                    return EventResult::Ignored;
                };
                if path.exists() {
                    EventResult::Command(HomeAction::Open(path))
                } else {
                    EventResult::Command(HomeAction::RemoveRecent(path))
                }
            }
            KeyCode::Delete | KeyCode::Backspace => self
                .recent_list
                .selected_path()
                .map_or(EventResult::Ignored, |path| {
                    EventResult::Command(HomeAction::RemoveRecent(path))
                }),
            _ => EventResult::Ignored,
        }
    }

    fn handle_open_table_key(&mut self, key: KeyEvent) -> EventResult<HomeAction> {
        match key.code {
            KeyCode::Enter => {
                let name = table_name(&self.open_input);
                EventResult::Command(HomeAction::Open(resolve_table_path(&name, &self.cwd)))
            }
            KeyCode::Backspace => {
                self.open_input.delete_char();
                EventResult::Handled
            }
            KeyCode::Char(c) => {
                self.open_input.insert_char(c);
                EventResult::Handled
            }
            _ => EventResult::Ignored,
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

        for (index, tab) in TABS.iter().enumerate() {
            let style = if index == self.selected_tab {
                selected_style
            } else {
                unselected_style
            };
            frame.render_widget(
                Paragraph::new(Line::from(tab.label()))
                    .centered()
                    .style(style),
                chunks[index],
            );
        }
    }

    fn render_current_tab(&self, frame: &mut Frame, area: Rect) {
        match self.current_tab() {
            HomeTab::Recent => self.recent_list.render(frame, area, self.theme),
            HomeTab::OpenTable => self.render_open_table(frame, area),
            HomeTab::Settings => {
                frame.render_widget(
                    Paragraph::new("设置功能即将上线")
                        .centered()
                        .fg(self.theme.text_dim),
                    area,
                );
            }
        }
    }

    fn render_open_table(&self, frame: &mut Frame, area: Rect) {
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

        self.open_input.render(frame, input_centered[1]);

        frame.render_widget(
            Paragraph::new("输入后按 Enter 打开或创建表格")
                .centered()
                .fg(self.theme.text_dim),
            hint_area,
        );
    }
}

impl HomeTab {
    fn label(self) -> &'static str {
        match self {
            HomeTab::Recent => "最近打开",
            HomeTab::OpenTable => "打开表格",
            HomeTab::Settings => "设置",
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
