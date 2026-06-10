use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
};
use unicode_width::UnicodeWidthStr;

use crate::theme::Theme;

pub struct Input {
    buffer: String,
    theme: Theme,
}

impl Input {
    pub fn new(theme: Theme) -> Self {
        Self {
            buffer: String::new(),
            theme,
        }
    }

    pub fn insert_char(&mut self, c: char) {
        self.buffer.push(c);
    }

    pub fn delete_char(&mut self) {
        self.buffer.pop();
    }

    pub fn buffer(&self) -> &str {
        &self.buffer
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let label_text = "表格路径  ";
        let label = Span::styled(label_text, Style::default().fg(self.theme.accent));

        let value_span = if self.buffer.is_empty() {
            Span::styled(
                "输入路径或名称...",
                Style::default().fg(self.theme.text_dim),
            )
        } else {
            Span::styled(&self.buffer, Style::default().fg(self.theme.text))
        };

        frame.render_widget(Paragraph::new(Line::from(vec![label, value_span])), area);

        frame.set_cursor_position((
            area.x + label_text.width() as u16 + self.buffer.as_str().width() as u16,
            area.y,
        ));
    }
}
