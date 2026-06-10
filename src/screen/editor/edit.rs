use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::Style,
    text::Line,
    widgets::Paragraph,
};

use crate::screen::EventResult;

use super::{
    mode::{EditorIntent, EditorReadModel, EditorView, FooterLine, Mode, ModeKind, ModeResult},
    navigation::NavigationMode,
};

/// 编辑模式
pub struct EditMode {
    buffer: String,
}

impl EditMode {
    pub fn new(initial: String, initial_char: Option<char>) -> Self {
        let mut buffer = initial;
        if let Some(c) = initial_char {
            buffer.push(c);
        }
        Self { buffer }
    }
}

impl Mode for EditMode {
    fn kind(&self) -> ModeKind {
        ModeKind::Edit
    }

    fn handle_key(&mut self, _view: EditorView<'_>, key: KeyEvent) -> ModeResult {
        match key.code {
            KeyCode::Enter => EventResult::Command(EditorIntent::CommitEdit(self.buffer.clone())),
            KeyCode::Esc => {
                EventResult::Command(EditorIntent::SwitchMode(Box::new(NavigationMode)))
            }
            KeyCode::Backspace => {
                self.buffer.pop();
                EventResult::Handled
            }
            KeyCode::Char(c) => {
                self.buffer.push(c);
                EventResult::Handled
            }
            _ => EventResult::Handled,
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect, read: EditorReadModel<'_>) -> Rect {
        use ratatui::text::Span;

        let [table_area, edit_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);

        let mut spans = vec![Span::styled(
            "编辑: ",
            Style::default().fg(read.theme.accent),
        )];
        if self.buffer.is_empty() {
            spans.push(Span::styled(
                "(空)",
                Style::default().fg(read.theme.text_dim),
            ));
        } else {
            spans.push(Span::styled(
                self.buffer.as_str(),
                Style::default().fg(read.theme.text),
            ));
            let cursor_char = if read.blink_visible { "█" } else { " " };
            spans.push(Span::styled(
                cursor_char,
                Style::default().fg(read.theme.text),
            ));
        }

        frame.render_widget(Paragraph::new(Line::from(spans)), edit_area);
        table_area
    }

    fn edit_buffer(&self) -> Option<&str> {
        Some(&self.buffer)
    }

    fn footer(&self, read: EditorReadModel<'_>) -> FooterLine {
        use ratatui::text::Span;
        FooterLine {
            hint: Some(Line::from(vec![
                Span::styled("Enter", Style::default().fg(read.theme.accent)),
                Span::styled(" 确认", Style::default().fg(read.theme.text_dim)),
                Span::styled("  ", Style::default().fg(read.theme.text_dim)),
                Span::styled("Esc", Style::default().fg(read.theme.accent)),
                Span::styled(" 取消", Style::default().fg(read.theme.text_dim)),
            ])),
            status: Some(Line::from(vec![
                Span::styled("[", Style::default().fg(read.theme.text_dim)),
                Span::styled(
                    read.viewport.cursor().display(),
                    Style::default().fg(read.theme.accent),
                ),
                Span::styled(", 编辑模式", Style::default().fg(read.theme.text_dim)),
                Span::styled("]", Style::default().fg(read.theme.text_dim)),
            ])),
        }
    }
}
