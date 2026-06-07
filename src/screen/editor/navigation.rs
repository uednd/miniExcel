use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{Frame, layout::Rect, style::Style, text::Line};

use super::{
    context::TableContext,
    mode::{Mode, ModeAction, ModeKind},
};

enum NavigationKey {
    Up,
    Down,
    Left,
    Right,
}

/// 默认模式 —— 光标导航、单元格删除、进入编辑。
pub struct NavigationMode;

impl Mode for NavigationMode {
    fn kind(&self) -> ModeKind {
        ModeKind::Navigation
    }

    fn handle_key(&mut self, ctx: &mut TableContext, key: KeyEvent) -> ModeAction {
        if let Some(nav) = Self::parse_direction(key) {
            match nav {
                NavigationKey::Up if ctx.cursor_row > 0 => ctx.cursor_row -= 1,
                NavigationKey::Down if ctx.cursor_row + 1 < ctx.wb.rows => ctx.cursor_row += 1,
                NavigationKey::Left if ctx.cursor_col > 0 => ctx.cursor_col -= 1,
                NavigationKey::Right if ctx.cursor_col + 1 < ctx.wb.columns => ctx.cursor_col += 1,
                _ => {}
            }
            ctx.scroll_into_view();
            return ModeAction::Nothing;
        }

        match key.code {
            KeyCode::Enter => ModeAction::SwitchToEdit { initial_char: None },
            KeyCode::Char(c) => ModeAction::SwitchToEdit {
                initial_char: Some(c),
            },
            KeyCode::Backspace | KeyCode::Delete => {
                ctx.wb.set_cell(
                    (ctx.cursor_col, ctx.cursor_row),
                    String::new(),
                    crate::model::cell::CellValue::Empty,
                );
                ModeAction::Nothing
            }
            _ => ModeAction::Nothing,
        }
    }

    fn render_frame(&self, _frame: &mut Frame, area: Rect, _ctx: &TableContext) -> Rect {
        area
    }

    fn footer_hint(&self, ctx: &TableContext) -> Option<Line<'static>> {
        use ratatui::text::Span;
        Some(Line::from(vec![
            Span::styled("Ctrl+S", Style::default().fg(ctx.theme.accent)),
            Span::styled(" 保存", Style::default().fg(ctx.theme.text_dim)),
            Span::styled("  ", Style::default().fg(ctx.theme.text_dim)),
            Span::styled("Ctrl+P", Style::default().fg(ctx.theme.accent)),
            Span::styled(" 菜单", Style::default().fg(ctx.theme.text_dim)),
            Span::styled("  ", Style::default().fg(ctx.theme.text_dim)),
            Span::styled("Enter", Style::default().fg(ctx.theme.accent)),
            Span::styled(" 编辑", Style::default().fg(ctx.theme.text_dim)),
        ]))
    }

    fn footer_status(&self, ctx: &TableContext) -> Option<Line<'static>> {
        use ratatui::text::Span;
        Some(Line::from(vec![
            Span::styled("[", Style::default().fg(ctx.theme.text_dim)),
            Span::styled(
                crate::model::cell::display_coord(ctx.cursor_row, ctx.cursor_col),
                Style::default().fg(ctx.theme.accent),
            ),
            Span::styled(", 导航模式", Style::default().fg(ctx.theme.text_dim)),
            Span::styled("]", Style::default().fg(ctx.theme.text_dim)),
        ]))
    }
}

impl NavigationMode {
    fn parse_direction(key: KeyEvent) -> Option<NavigationKey> {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            return None;
        }
        match key.code {
            KeyCode::Up => Some(NavigationKey::Up),
            KeyCode::Down => Some(NavigationKey::Down),
            KeyCode::Left => Some(NavigationKey::Left),
            KeyCode::Right => Some(NavigationKey::Right),
            _ => None,
        }
    }
}
