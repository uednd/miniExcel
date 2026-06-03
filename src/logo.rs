//! ASCII 艺术字 logo 模块。
//!
//! 将 "miniExcel" 词标渲染为双色 ASCII 艺术字横幅，并居中显示在终端窗口中。

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

/// 在给定区域内居中渲染 "miniExcel" ASCII 艺术字 logo。
///
/// 返回 logo 占据的 [`Rect`]，调用方可据此定位其他 UI 元素（如退出提示）。
pub fn render(frame: &mut Frame, area: Rect) -> Rect {
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

    let art_height = art_lines.len() as u16;

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

    art_area
}
