//! ASCII 艺术字 logo 模块。

use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::Paragraph,
};

const MINI_COLOR: Color = Color::Rgb(135, 142, 142);
const MINI_ART: &[&str] = &[
    "      ▄      ▄ ",
    "▄▄▄▄  ▄ ▄▄▄  ▄ ",
    "█ █ █ █ █  █ █ ",
    "▀ ▀ ▀ ▀ ▀  ▀ ▀ ",
];

const EXCEL_COLOR: Color = Color::Rgb(220, 224, 224);
const EXCEL_ART: &[&str] = &[
    "▄▄▄▄                █ ",
    "█▄▄▄ ▄  ▄ █▀▀▀ █▀▀█ █ ",
    "█     ▄▀  █    █▀▀▀ █ ",
    "▀▀▀▀ ▀  ▀ ▀▀▀▀ ▀▀▀▀ ▀▀",
];

pub const LOGO_HEIGHT: u16 = MINI_ART.len() as u16;

pub struct Logo;

impl Logo {
    /// 创建 Logo 组件实例。
    pub fn new() -> Self {
        Self
    }

    /// 渲染项目 Logo。
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let art_lines: Vec<Line> = MINI_ART
            .iter()
            .zip(EXCEL_ART)
            .map(|(mini, excel)| {
                Line::from(vec![
                    Span::styled(*mini, Style::default().fg(MINI_COLOR)),
                    Span::styled(*excel, Style::default().fg(EXCEL_COLOR)),
                ])
            })
            .collect();

        let art_height = LOGO_HEIGHT;

        let art_width = art_lines
            .iter()
            .map(|l| l.width() as u16)
            .max()
            .unwrap_or(0);

        let art_area = area.centered(
            Constraint::Length(art_width),
            Constraint::Length(art_height),
        );

        let paragraph = Paragraph::new(Text::from(art_lines));
        frame.render_widget(paragraph, art_area);
    }
}
