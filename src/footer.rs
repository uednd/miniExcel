//! 底栏渲染模块。
//!
//! 在终端底部分别显示当前工作目录路径和应用版本号，

use ratatui::{
    Frame,
    layout::Rect,
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
        let bottom_row = Rect::new(
            area.x,
            area.y + area.height.saturating_sub(1),
            area.width,
            1,
        );

        let footer_style = Style::default().fg(FOOTER_COLOR);

        frame.render_widget(
            Paragraph::new(Line::from(self.current_dir.as_str()).style(footer_style)),
            bottom_row,
        );

        let version_x = area.width.saturating_sub(self.version.len() as u16);
        let version_area = Rect::new(
            area.x + version_x,
            bottom_row.y,
            self.version.len() as u16,
            1,
        );

        frame.render_widget(
            Paragraph::new(Line::from(self.version.as_str()).style(footer_style)),
            version_area,
        );
    }
}
