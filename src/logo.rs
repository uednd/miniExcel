//! ASCII 艺术字 logo 模块。
//!
//! 将 "mini excel" 词标渲染为双色 ASCII 艺术字横幅，并居中显示在终端窗口中。

use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::Paragraph,
};

const ASCII_ART: &str = concat!(
    "         █         █  █████                    █\n",
    "                      █                        █\n",
    "██ ███  ██  ████  ██  █      █   █  ████  ████ █\n",
    "█ █ █ █  █  █  █   █  ████    █ █  █     █  █  █\n",
    "█ █ █ █  █  █  █   █  █        █   █     ████  █\n",
    "█   █ █  █  █  █   █  █       █ █  █     █     █\n",
    "█   █ █ ██  █  █  ██  █████  █   █  ████  ████ ██\n",
);

/// 每行拆分为 "mini" 和 "excel" 两部分的字符位置。
const SPLIT_AT: usize = 22;

/// logo 中 "mini" 部分的前景色。
const MINI_COLOR: Color = Color::Rgb(135, 142, 142);

/// logo 中 "excel" 部分的前景色。
const EXCEL_COLOR: Color = Color::Rgb(220, 224, 224);

/// 在指定字符位置（非字节位置）拆分字符串。
fn split_at_char(s: &str, char_count: usize) -> (&str, &str) {
    let split = s
        .char_indices()
        .nth(char_count)
        .map(|(index, _)| index)
        .unwrap_or(s.len());
    s.split_at(split)
}

/// 在给定区域内居中渲染 "mini excel" ASCII 艺术字 logo。
///
/// 返回 logo 占据的 [`Rect`]，调用方可据此定位其他 UI 元素（如退出提示）。
pub fn render(frame: &mut Frame, area: Rect) -> Rect {
    let art_lines: Vec<Line> = ASCII_ART
        .lines()
        .map(|s| {
            let (mini, excel) = split_at_char(s, SPLIT_AT);
            Line::from(vec![
                Span::styled(mini, Style::default().fg(MINI_COLOR)),
                Span::styled(excel, Style::default().fg(EXCEL_COLOR)),
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
