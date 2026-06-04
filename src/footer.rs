//! 底栏渲染模块。
//!
//! 在终端底部使用 Layout 横向分割，左侧显示路径、右侧显示版本号。

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Padding, Paragraph},
};

use crate::theme::{TEXT_DIM, TEXT_HIGHLIGHT, THEME_GREEN};

/// 终端底栏组件。
pub struct Footer {
    current_dir: String,
    version: String,
}

impl Footer {
    pub fn new(current_dir: String, version: String) -> Self {
        Self {
            current_dir,
            version,
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let block = Block::new().padding(Padding::new(2, 2, 0, 1));

        let [path_area, hint_area, version_area] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
        ])
        .areas(block.inner(area));

        let dim_style = Style::default().fg(TEXT_DIM);
        let theme_style = Style::default().fg(THEME_GREEN);
        let highlight_style = Style::default().fg(TEXT_HIGHLIGHT);

        let hint = Line::from(vec![
            Span::styled("● 提示", theme_style),
            Span::styled(" 使用 ", dim_style),
            Span::styled("←/→", highlight_style),
            Span::styled(" 选择菜单", dim_style),
        ]);

        frame.render_widget(
            Paragraph::new(Line::from(self.current_dir.as_str()).style(dim_style)),
            path_area,
        );
        frame.render_widget(Paragraph::new(hint).alignment(Alignment::Center), hint_area);
        frame.render_widget(
            Paragraph::new(Line::from(self.version.as_str()).style(dim_style))
                .alignment(Alignment::Right),
            version_area,
        );
    }
}
