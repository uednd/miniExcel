//! 主菜单/导航入口渲染模块。
//!
//! 横向排列「新建表格」「最近使用」「设置」三个入口，
//! 通过样式变化标识当前选中项。

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::Paragraph,
};

/// 选中项背景色。
const THEME_GREEN: Color = Color::Rgb(80, 160, 100);
/// 选中项文字色。
const SELECTED_TEXT: Color = Color::Rgb(16, 32, 22);
/// 未选中项文字色。
const UNSELECTED_TEXT: Color = Color::Rgb(160, 160, 160);
/// 菜单项标签。
const MENU_ITEMS: [&str; 3] = ["新建表格", "最近打开", "设置"];

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
        if self.selected + 1 < MENU_ITEMS.len() {
            self.selected += 1;
        }
    }

    /// 在给定区域内居中渲染横向菜单。
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let menu_area = area.centered_horizontally(Constraint::Percentage(50));
        let chunks: [Rect; MENU_ITEMS.len()] =
            Layout::horizontal([Constraint::Fill(1); MENU_ITEMS.len()]).areas(menu_area);

        let selected_style = Style::default()
            .fg(SELECTED_TEXT)
            .bg(THEME_GREEN)
            .add_modifier(Modifier::BOLD);
        let unselected_style = Style::default().fg(UNSELECTED_TEXT);

        for (i, label) in MENU_ITEMS.iter().enumerate() {
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
