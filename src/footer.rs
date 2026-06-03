//! 底栏渲染模块。
//!
//! 在终端底部使用 Layout 横向分割，左侧显示路径、右侧显示版本号。

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::Paragraph,
};

/// 底栏文字颜色。
const FOOTER_COLOR: Color = Color::Rgb(160, 160, 160);

/// 终端底栏组件。
pub struct Footer {
    current_dir: String,
    version: String,
}

impl Footer {
    /// 创建底栏实例，保存路径和版本号字符串。
    pub fn new(current_dir: String, version: String) -> Self {
        Self {
            current_dir,
            version,
        }
    }

    /// 在终端底部渲染底栏。
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let [path_area, version_area] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(self.version.len() as u16),
        ])
        .areas(area);

        let footer_style = Style::default().fg(FOOTER_COLOR);

        frame.render_widget(
            Paragraph::new(Line::from(self.current_dir.as_str()).style(footer_style)),
            path_area,
        );
        frame.render_widget(
            Paragraph::new(Line::from(self.version.as_str()).style(footer_style)),
            version_area,
        );
    }
}
