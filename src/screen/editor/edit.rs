use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::Style,
    text::Line,
    widgets::Paragraph,
};

use crate::{
    model::cell::CellValue,
    util::cursor_span,
};

use super::{
    context::TableContext,
    mode::{Mode, ModeAction, ModeKind},
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

    fn handle_key(&mut self, ctx: &mut TableContext, key: KeyEvent) -> ModeAction {
        match key.code {
            KeyCode::Enter => {
                if !self.buffer.is_empty() {
                    ctx.wb.set_cell(
                        ctx.cursor,
                        self.buffer.clone(),
                        CellValue::Text(self.buffer.clone()),
                    );
                }
                ModeAction::SwitchToNavigation
            }
            KeyCode::Esc => ModeAction::SwitchToNavigation,
            KeyCode::Backspace => {
                self.buffer.pop();
                ModeAction::Nothing
            }
            KeyCode::Char(c) => {
                self.buffer.push(c);
                ModeAction::Nothing
            }
            _ => ModeAction::Nothing,
        }
    }

    fn render_frame(&self, frame: &mut Frame, area: Rect, ctx: &TableContext) -> Rect {
        use ratatui::text::Span;

        let [table_area, edit_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);

        let mut spans = vec![Span::styled(
            "编辑: ",
            Style::default().fg(ctx.theme.accent),
        )];
        if self.buffer.is_empty() {
            spans.push(Span::styled(
                "(空)",
                Style::default().fg(ctx.theme.text_dim),
            ));
        } else {
            spans.push(Span::styled(
                self.buffer.as_str(),
                Style::default().fg(ctx.theme.text),
            ));
            spans.push(cursor_span(Style::default().fg(ctx.theme.text)));
        }

        frame.render_widget(Paragraph::new(Line::from(spans)), edit_area);
        table_area
    }

    fn edit_buffer(&self) -> Option<&str> {
        Some(&self.buffer)
    }

    fn footer_hint(&self, ctx: &TableContext) -> Option<Line<'static>> {
        use ratatui::text::Span;
        Some(Line::from(vec![
            Span::styled("Enter", Style::default().fg(ctx.theme.accent)),
            Span::styled(" 确认", Style::default().fg(ctx.theme.text_dim)),
            Span::styled("  ", Style::default().fg(ctx.theme.text_dim)),
            Span::styled("Esc", Style::default().fg(ctx.theme.accent)),
            Span::styled(" 取消", Style::default().fg(ctx.theme.text_dim)),
        ]))
    }

    fn footer_status(&self, ctx: &TableContext) -> Option<Line<'static>> {
        use ratatui::text::Span;
        Some(Line::from(vec![
            Span::styled("[", Style::default().fg(ctx.theme.text_dim)),
            Span::styled(
                ctx.cursor.display(),
                Style::default().fg(ctx.theme.accent),
            ),
            Span::styled(", 编辑模式", Style::default().fg(ctx.theme.text_dim)),
            Span::styled("]", Style::default().fg(ctx.theme.text_dim)),
        ]))
    }
}
