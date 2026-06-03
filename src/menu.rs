//! 主菜单/导航入口渲染模块。
//!
//! 横向排列「新建表格」「最近使用」「设置」三个入口，
//! 通过样式变化标识当前选中项。

use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::Paragraph,
    Frame,
};
use unicode_width::UnicodeWidthStr;

/// 一些主题色。
const THEME_GREEN: Color = Color::Rgb(80, 160, 100);
const SELECTED_TEXT: Color = Color::Rgb(16, 32, 22);
const HORIZONTAL_PADDING: usize = 2;

/// 菜单项标签。
const MENU_ITEMS: &[&str] = &["新建表格", "最近使用", "设置"];

/// 菜单选中状态。
pub struct Menu {
    selected: usize,
}

impl Menu {
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

    /// 在给定区域内居中渲染横向菜单，返回菜单占用的 [`Rect`]。
    pub fn render(&self, frame: &mut Frame, area: Rect) -> Rect {
        let line = Line::from(menu_spans(self.selected));
        let width = line.width() as u16;

        let menu_area = area.centered(Constraint::Length(width), Constraint::Length(1));

        let paragraph = Paragraph::new(Text::from(line));
        frame.render_widget(paragraph, menu_area);

        menu_area
    }
}

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

fn menu_item_width() -> usize {
    MENU_ITEMS
        .iter()
        .map(|label| UnicodeWidthStr::width(*label))
        .max()
        .unwrap_or(0)
        + HORIZONTAL_PADDING * 2
}

fn centered_label(label: &str, width: usize) -> String {
    let label_width = UnicodeWidthStr::width(label);
    let padding = width.saturating_sub(label_width);
    let left = padding / 2;
    let right = padding - left;

    format!("{}{}{}", " ".repeat(left), label, " ".repeat(right))
}
