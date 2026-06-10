use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{Frame, layout::Rect, style::Style, text::Line};

use crate::screen::EventResult;

use super::{
    context::TableContext,
    mode::{
        Direction, EditorIntent, EditorView, FooterLine, Mode, ModeKind, ModeResult, Selection,
    },
};

/// 默认模式 —— 光标导航、单元格删除、进入编辑。
pub struct NavigationMode;

impl Mode for NavigationMode {
    fn kind(&self) -> ModeKind {
        ModeKind::Navigation
    }

    fn handle_key(&mut self, view: EditorView<'_>, key: KeyEvent) -> ModeResult {
        // --- 选中模式下的按键处理 ---
        if let Some(sel) = view.selection() {
            // Range 选中：Shift+方向键扩展选区
            if let Some(nav) = Self::parse_shift_direction(key)
                && let Selection::Range { .. } = sel
            {
                return EventResult::Command(EditorIntent::ExtendRangeSelection(nav));
            }

            match (key.code, &sel) {
                // Esc 退出选中
                (KeyCode::Esc, _) => {
                    return EventResult::Command(EditorIntent::ClearSelection);
                }
                // Delete/Backspace: 清空选中内容
                (KeyCode::Delete | KeyCode::Backspace, _) => {
                    return EventResult::Command(EditorIntent::ClearSelectionCells);
                }
                // 同快捷键再次按退出选中（仅 Row/Column）
                (KeyCode::Left | KeyCode::Right, Selection::Row(_))
                    if key
                        .modifiers
                        .contains(KeyModifiers::CONTROL | KeyModifiers::SHIFT) =>
                {
                    return EventResult::Command(EditorIntent::ClearSelection);
                }
                (KeyCode::Up | KeyCode::Down, Selection::Column(_))
                    if key
                        .modifiers
                        .contains(KeyModifiers::CONTROL | KeyModifiers::SHIFT) =>
                {
                    return EventResult::Command(EditorIntent::ClearSelection);
                }
                // 方向键退出选中并执行移动
                _ if Self::parse_direction(key).is_some() => {
                    let nav = Self::parse_direction(key).unwrap();
                    return EventResult::Command(EditorIntent::MoveCursorAndClearSelection(nav));
                }
                _ => return EventResult::Handled,
            }
        }

        // --- 非选中模式：Shift+方向键创建 Range 选区 ---
        if let Some(nav) = Self::parse_shift_direction(key) {
            return EventResult::Command(EditorIntent::StartRangeSelection(nav));
        }

        // --- 非选中模式：方向键 ---
        if let Some(nav) = Self::parse_direction(key) {
            return EventResult::Command(EditorIntent::MoveCursor(nav));
        }

        // --- 非选中模式：选区快捷键（Ctrl+Shift+方向键） ---
        match key.code {
            KeyCode::Left | KeyCode::Right
                if key
                    .modifiers
                    .contains(KeyModifiers::CONTROL | KeyModifiers::SHIFT) =>
            {
                return EventResult::Command(EditorIntent::SelectCurrentRow);
            }
            KeyCode::Up | KeyCode::Down
                if key
                    .modifiers
                    .contains(KeyModifiers::CONTROL | KeyModifiers::SHIFT) =>
            {
                return EventResult::Command(EditorIntent::SelectCurrentColumn);
            }
            _ => {}
        }

        // --- 原有逻辑 ---
        match key.code {
            KeyCode::Enter => EventResult::Command(EditorIntent::StartEdit { initial_char: None }),
            KeyCode::Char(c) => EventResult::Command(EditorIntent::StartEdit {
                initial_char: Some(c),
            }),
            KeyCode::Backspace | KeyCode::Delete => {
                EventResult::Command(EditorIntent::ClearCurrentCell)
            }
            _ => EventResult::Handled,
        }
    }

    fn render(&self, _frame: &mut Frame, area: Rect, _ctx: &TableContext) -> Rect {
        area
    }

    fn footer(&self, ctx: &TableContext) -> FooterLine {
        use ratatui::text::Span;
        let hint = if let Some(stats) = ctx.selection_stats() {
            Line::from(vec![
                Span::styled("平均值=", Style::default().fg(ctx.theme.accent)),
                Span::styled(
                    format!(" {}", format_number(stats.average)),
                    Style::default().fg(ctx.theme.text_dim),
                ),
                Span::styled("  ", Style::default().fg(ctx.theme.text_dim)),
                Span::styled("求和=", Style::default().fg(ctx.theme.accent)),
                Span::styled(
                    format!(" {}", format_number(stats.sum)),
                    Style::default().fg(ctx.theme.text_dim),
                ),
                Span::styled("  ", Style::default().fg(ctx.theme.text_dim)),
                Span::styled("计数=", Style::default().fg(ctx.theme.accent)),
                Span::styled(
                    format!(" {}", stats.count),
                    Style::default().fg(ctx.theme.text_dim),
                ),
            ])
        } else {
            Line::from(vec![
                Span::styled("Ctrl+S", Style::default().fg(ctx.theme.accent)),
                Span::styled(" 保存", Style::default().fg(ctx.theme.text_dim)),
                Span::styled("  ", Style::default().fg(ctx.theme.text_dim)),
                Span::styled("Ctrl+P", Style::default().fg(ctx.theme.accent)),
                Span::styled(" 菜单", Style::default().fg(ctx.theme.text_dim)),
                Span::styled("  ", Style::default().fg(ctx.theme.text_dim)),
                Span::styled("Enter", Style::default().fg(ctx.theme.accent)),
                Span::styled(" 编辑", Style::default().fg(ctx.theme.text_dim)),
            ])
        };

        FooterLine {
            hint: Some(hint),
            status: Some(Line::from(vec![
                Span::styled("[", Style::default().fg(ctx.theme.text_dim)),
                Span::styled(
                    ctx.cursor().display(),
                    Style::default().fg(ctx.theme.accent),
                ),
                Span::styled(", 光标模式", Style::default().fg(ctx.theme.text_dim)),
                Span::styled("]", Style::default().fg(ctx.theme.text_dim)),
            ])),
        }
    }
}

impl NavigationMode {
    fn parse_direction(key: KeyEvent) -> Option<Direction> {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            return None;
        }
        match key.code {
            KeyCode::Up => Some(Direction::Up),
            KeyCode::Down => Some(Direction::Down),
            KeyCode::Left => Some(Direction::Left),
            KeyCode::Right => Some(Direction::Right),
            _ => None,
        }
    }

    /// 仅捕获 Shift+方向键（不含 Ctrl），用于框选。
    fn parse_shift_direction(key: KeyEvent) -> Option<Direction> {
        if !key.modifiers.contains(KeyModifiers::SHIFT)
            || key.modifiers.contains(KeyModifiers::CONTROL)
        {
            return None;
        }
        match key.code {
            KeyCode::Up => Some(Direction::Up),
            KeyCode::Down => Some(Direction::Down),
            KeyCode::Left => Some(Direction::Left),
            KeyCode::Right => Some(Direction::Right),
            _ => None,
        }
    }
}

fn format_number(n: f64) -> String {
    if n == 0.0 {
        return "0".to_string();
    }

    let abs = n.abs();
    if !(1e-6..1e12).contains(&abs) {
        return format!("{:.2e}", n);
    }

    let text = format!("{:.10}", n);
    let text = text.trim_end_matches('0');
    text.trim_end_matches('.').to_string()
}
