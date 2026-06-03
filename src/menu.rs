//! 主菜单/导航入口渲染模块。
//!
//! 横向排列「新建表格」「最近使用」「设置」三个入口，
//! 通过样式变化标识当前选中项。

use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::Paragraph,
};
use unicode_width::UnicodeWidthStr;

use crate::logo::LOGO_HEIGHT;

/// logo 底部到菜单顶部的间距。
pub const LOGO_MENU_GAP: u16 = 2;
/// 选中项背景色。
const THEME_GREEN: Color = Color::Rgb(80, 160, 100);
/// 选中项文字色。
const SELECTED_TEXT: Color = Color::Rgb(16, 32, 22);
/// 菜单项左右两侧的内边距（字符数）。
const HORIZONTAL_PADDING: usize = 2;
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

    /// 在 logo 下方居中渲染横向菜单。
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let line = Line::from(menu_spans(self.selected));
        let width = line.width() as u16;

        let logo_bottom = area.y + (area.height.saturating_sub(LOGO_HEIGHT)) / 2 + LOGO_HEIGHT;
        let menu_y = logo_bottom + LOGO_MENU_GAP;
        let menu_origin = Rect::new(area.x, menu_y, area.width, 1);
        let menu_area = menu_origin.centered(Constraint::Length(width), Constraint::Length(1));

        let paragraph = Paragraph::new(Text::from(line));
        frame.render_widget(paragraph, menu_area);
    }
}

/// 构建菜单项的 [`Span`] 列表，高亮当前选中项。
fn menu_spans(selected: usize) -> Vec<Span<'static>> {
    let item_width = menu_item_width();

    MENU_ITEMS
        .iter()
        .enumerate()
        .map(|(i, label)| {
            let style = if i == selected {
                Style::default()
                    .fg(SELECTED_TEXT)
                    .bg(THEME_GREEN)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Rgb(160, 160, 160))
            };

            Span::styled(centered_label(label, item_width), style)
        })
        .collect()
}

/// 计算绘制单个菜单项所需的字符宽度。
fn menu_item_width() -> usize {
    MENU_ITEMS
        .iter()
        .map(|label| UnicodeWidthStr::width(*label))
        .max()
        .unwrap_or(0)
        + HORIZONTAL_PADDING * 2
}

/// 将标签文本在指定宽度内居中填充。
fn centered_label(label: &str, width: usize) -> String {
    let label_width = UnicodeWidthStr::width(label);
    let padding = width.saturating_sub(label_width);
    let left = padding / 2;
    let right = padding - left;

    format!("{}{}{}", " ".repeat(left), label, " ".repeat(right))
}
