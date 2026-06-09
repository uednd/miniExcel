use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::Style,
    text::Line,
    widgets::{List, ListItem},
    Frame,
};

use crate::{
    screen::editor::{ModeAction, TableContext},
    theme::Theme,
};

/// 可选项
pub struct SelectableItem {
    pub label: String,
    pub action: Box<dyn Fn(&mut TableContext) -> ModeAction>,
}

impl SelectableItem {
    pub fn new(
        label: impl Into<String>,
        action: impl Fn(&mut TableContext) -> ModeAction + 'static,
    ) -> Self {
        Self {
            label: label.into(),
            action: Box::new(action),
        }
    }
}

/// 通用选择列表面板
pub struct SelectableList {
    items: Vec<SelectableItem>,
    selected: usize,
}

impl SelectableList {
    /// 创建一个选择列表，默认选中第 0 项。
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

    /// 执行当前选中项的 action 闭包，返回 ModeAction。
    pub fn handle_enter(&self, ctx: &mut TableContext) -> Option<ModeAction> {
        self.items
            .get(self.selected)
            .map(|item| (item.action)(ctx))
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
