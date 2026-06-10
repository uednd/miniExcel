use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::Style,
    text::Line,
    widgets::{List, ListItem},
};

use crate::{screen::editor::ModeResult, theme::Theme};

/// 编辑器命令列表中的一项。
///
/// `action` 返回编辑器意图；具体状态修改由编辑器会话统一执行。
pub struct SelectableItem {
    pub label: String,
    pub action: Box<dyn Fn() -> ModeResult>,
}

impl SelectableItem {
    /// 创建一个带标签和执行动作的列表项。
    pub fn new(label: impl Into<String>, action: impl Fn() -> ModeResult + 'static) -> Self {
        Self {
            label: label.into(),
            action: Box::new(action),
        }
    }
}

/// 编辑器命令选择列表。
///
/// 该列表负责选中项状态和渲染，按下 Enter 时执行当前项的动作。
pub struct SelectableList {
    items: Vec<SelectableItem>,
    selected: usize,
}

impl SelectableList {
    /// 创建一个选择列表，默认选中第 0 项。
    ///
    /// 调用者应传入至少一个列表项。
    pub fn new(items: Vec<SelectableItem>) -> Self {
        Self { items, selected: 0 }
    }

    /// 向上移动选中项。
    pub fn handle_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    /// 向下移动选中项。
    pub fn handle_down(&mut self) {
        if self.selected + 1 < self.items.len() {
            self.selected += 1;
        }
    }

    /// 执行当前选中项的动作。
    ///
    /// 如果列表为空，返回 `None`。
    pub fn handle_enter(&self) -> Option<ModeResult> {
        self.items.get(self.selected).map(|item| (item.action)())
    }

    /// 在指定区域内渲染选择列表。
    pub fn render(&self, frame: &mut Frame, area: Rect, theme: Theme) {
        let [_, list_area, _] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(self.items.len() as u16 * 2),
            Constraint::Fill(1),
        ])
        .areas(area);

        let list_items: Vec<ListItem> = self
            .items
            .iter()
            .map(|item| {
                ListItem::new(Line::from(item.label.as_str()))
                    .style(Style::default().fg(theme.text_dim))
            })
            .collect();

        let list = List::new(list_items)
            .highlight_symbol(ratatui::text::Span::styled(
                "▎",
                Style::default().fg(theme.accent),
            ))
            .highlight_style(Style::default().fg(theme.text).bg(theme.surface));

        let mut state = ratatui::widgets::ListState::default();
        state.select(Some(self.selected));

        frame.render_stateful_widget(list, list_area, &mut state);
    }
}
