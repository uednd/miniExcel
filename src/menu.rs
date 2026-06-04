//! 主菜单/导航入口渲染模块。
//!
//! 横向排列「新建表格」「最近使用」「设置」三个入口，
//! 通过样式变化标识当前选中项。每个入口关联一个 `MenuAction`，
//! 调用方通过 `selected_action()` 获取当前选中项的动作语义。

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::Paragraph,
};

use crate::theme::{TEXT_HIGHLIGHT, TEXT_SELECTED, THEME_GREEN};

/// 菜单动作。
pub enum MenuAction {
    NewTable,
    Recent,
    Settings,
}

/// 菜单项：(标签, 动作)。
const MENU_ENTRIES: [(&str, MenuAction); 3] = [
    ("新建表格", MenuAction::NewTable),
    ("最近打开", MenuAction::Recent),
    ("设置", MenuAction::Settings),
];

/// 菜单选中状态。
pub struct Menu {
    selected: usize,
}

impl Menu {
    /// 创建菜单实例，默认选中第一项。
    pub fn new() -> Self {
        Self { selected: 0 }
    }

    /// 向左移动选中项（`←`）。
    pub fn move_left(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    /// 向右移动选中项（`→`）。
    pub fn move_right(&mut self) {
        if self.selected + 1 < MENU_ENTRIES.len() {
            self.selected += 1;
        }
    }

    /// 返回当前选中项对应的动作。
    pub fn selected_action(&self) -> &MenuAction {
        &MENU_ENTRIES[self.selected].1
    }

    /// 在给定区域内居中渲染横向菜单。
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let menu_area = area.centered_horizontally(Constraint::Percentage(50));
        let chunks: [Rect; MENU_ENTRIES.len()] =
            Layout::horizontal([Constraint::Fill(1); MENU_ENTRIES.len()]).areas(menu_area);

        let selected_style = Style::default()
            .fg(TEXT_SELECTED)
            .bg(THEME_GREEN)
            .add_modifier(Modifier::BOLD);
        let unselected_style = Style::default().fg(TEXT_HIGHLIGHT);

        for (i, (label, _)) in MENU_ENTRIES.iter().enumerate() {
            let style = if i == self.selected {
                selected_style
            } else {
                unselected_style
            };
            frame.render_widget(
                Paragraph::new(Line::from(*label)).centered().style(style),
                chunks[i],
            );
        }
    }
}
