use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::Style,
    text::Line,
    widgets::Paragraph,
};

use crate::{model::cell::CellValue, util::cursor_span};

use super::{
    context::TableContext,
    mode::{FooterLine, Mode, ModeAction, ModeKind},
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

    fn handle_key(&mut self, ctx: &mut TableContext, key: KeyEvent) -> ModeAction {
        match key.code {
            KeyCode::Enter => {
                if !self.buffer.is_empty() {
                    ctx.wb.set_cell(
                        ctx.viewport.cursor(),
                        self.buffer.clone(),
                        CellValue::Text(self.buffer.clone()),
                    );
                }
                ModeAction::SwitchMode(Box::new(NavigationMode))
            }
            KeyCode::Esc => ModeAction::SwitchMode(Box::new(NavigationMode)),
            KeyCode::Backspace => {
                self.buffer.pop();
                ModeAction::Handled
            }
            KeyCode::Char(c) => {
                self.buffer.push(c);
                ModeAction::Handled
            }
            _ => ModeAction::Handled,
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect, ctx: &TableContext) -> Rect {
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

    fn footer(&self, ctx: &TableContext) -> FooterLine {
        use ratatui::text::Span;
        FooterLine {
            hint: Some(Line::from(vec![
                Span::styled("Enter", Style::default().fg(ctx.theme.accent)),
                Span::styled(" 确认", Style::default().fg(ctx.theme.text_dim)),
                Span::styled("  ", Style::default().fg(ctx.theme.text_dim)),
                Span::styled("Esc", Style::default().fg(ctx.theme.accent)),
                Span::styled(" 取消", Style::default().fg(ctx.theme.text_dim)),
            ])),
            status: Some(Line::from(vec![
                Span::styled("[", Style::default().fg(ctx.theme.text_dim)),
                Span::styled(
                    ctx.viewport.cursor().display(),
                    Style::default().fg(ctx.theme.accent),
                ),
                Span::styled(", 编辑模式", Style::default().fg(ctx.theme.text_dim)),
                Span::styled("]", Style::default().fg(ctx.theme.text_dim)),
            ])),
        }
    }
}
