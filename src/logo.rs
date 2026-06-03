//! ASCII 艺术字 logo 模块。

use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::Paragraph,
};

const MINI_COLOR: Color = Color::Rgb(135, 142, 142);
const MINI_ART: [&str; 4] = [
    "      ▄      ▄ ",
    "▄▄▄▄  ▄ ▄▄▄  ▄ ",
    "█ █ █ █ █  █ █ ",
    "▀ ▀ ▀ ▀ ▀  ▀ ▀ ",
];

const EXCEL_COLOR: Color = Color::Rgb(220, 224, 224);
const EXCEL_ART: [&str; 4] = [
    "▄▄▄▄                █ ",
    "█▄▄▄ ▄  ▄ █▀▀▀ █▀▀█ █ ",
    "█     ▄▀  █    █▀▀▀ █ ",
    "▀▀▀▀ ▀  ▀ ▀▀▀▀ ▀▀▀▀ ▀▀",
];

/// logo ASCII 艺术字的高度（行数）。
pub const LOGO_HEIGHT: u16 = {
    let m = MINI_ART.len();
    let e = EXCEL_ART.len();
    if m > e { m } else { e }
} as u16;

/// 居中渲染 "miniExcel" ASCII 艺术字 logo 的组件。
pub struct Logo;

impl Logo {
    /// 创建 Logo 组件实例。
    pub fn new() -> Self {
        Self
    }

    /// 在给定区域内居中渲染 "miniExcel" ASCII 艺术字 logo。
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let art_lines: Vec<Line> = MINI_ART
            .into_iter()
            .zip(EXCEL_ART)
            .map(|(mini, excel)| {
                Line::from(vec![
                    Span::styled(mini, Style::default().fg(MINI_COLOR)),
                    Span::styled(excel, Style::default().fg(EXCEL_COLOR)),
                ])
            })
            .collect();

        let art_width = art_lines
            .iter()
            .map(|l| l.width() as u16)
            .max()
            .unwrap_or(0);

        let art_area = area.centered_horizontally(Constraint::Length(art_width));
        frame.render_widget(Paragraph::new(Text::from(art_lines)), art_area);
    }
}
