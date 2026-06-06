use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
};

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
        let label = Span::styled("表格名称  ", Style::default().fg(self.theme.accent));

        let (value_span, cursor) = if self.buffer.is_empty() {
            (
                Span::styled("输入表格名称...", Style::default().fg(self.theme.text_dim)),
                false,
            )
        } else {
            (
                Span::styled(&self.buffer, Style::default().fg(self.theme.text)),
                true,
            )
        };

        let mut spans = vec![label, value_span];

        if cursor {
            spans.push(Span::styled("█", Style::default().fg(self.theme.text)));
        }

        frame.render_widget(Paragraph::new(Line::from(spans)), area);
    }
}
