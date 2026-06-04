//! ASCII Logo 模块。

use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::Style,
    text::{Line, Span, Text},
    widgets::Paragraph,
};

use crate::theme::{LOGO_EXCEL, LOGO_MINI};

const MINI_ART: [&str; 4] = [
    "      ▄      ▄ ",
    "▄▄▄▄  ▄ ▄▄▄  ▄ ",
    "█ █ █ █ █  █ █ ",
    "▀ ▀ ▀ ▀ ▀  ▀ ▀ ",
];

const EXCEL_ART: [&str; 4] = [
    "▄▄▄▄                █ ",
    "█▄▄▄ ▄  ▄ █▀▀▀ █▀▀█ █ ",
    "█     ▄▀  █    █▀▀▀ █ ",
    "▀▀▀▀ ▀  ▀ ▀▀▀▀ ▀▀▀▀ ▀▀",
];

/// Logo 高度。
pub const LOGO_HEIGHT: u16 = {
    let m = MINI_ART.len();
    let e = EXCEL_ART.len();
    if m > e { m } else { e }
} as u16;

pub struct Logo;

impl Logo {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let art_lines: Vec<Line> = MINI_ART
            .into_iter()
            .zip(EXCEL_ART)
            .map(|(mini, excel)| {
                Line::from(vec![
                    Span::styled(mini, Style::default().fg(LOGO_MINI)),
                    Span::styled(excel, Style::default().fg(LOGO_EXCEL)),
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
