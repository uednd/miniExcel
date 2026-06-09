use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{Frame, layout::Rect, style::Style, text::Line};

use crate::model::workbook::ClearSpec;

use super::{
    context::TableContext,
    mode::{Mode, ModeAction, ModeKind, Selection},
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
        // --- 选中模式下的按键处理 ---
        if let Some(ref sel) = ctx.selection {
            // Range 选中：Shift+方向键扩展选区
            if let Some(nav) = Self::parse_shift_direction(key)
                && let Selection::Range { anchor, .. } = *sel
            {
                Self::apply_direction(ctx, nav);
                ctx.selection = Some(Selection::Range {
                    anchor,
                    cursor: ctx.cursor,
                });
                return ModeAction::Nothing;
            }

            match (key.code, sel) {
                // Esc 退出选中
                (KeyCode::Esc, _) => {
                    ctx.selection = None;
                    return ModeAction::Nothing;
                }
                // Delete/Backspace: 清空选中内容
                (KeyCode::Delete | KeyCode::Backspace, _) => {
                    let spec = match *sel {
                        Selection::Row(r) => ClearSpec::Row(r),
                        Selection::Column(c) => ClearSpec::Column(c),
                        Selection::Range { anchor, cursor } => {
                            ClearSpec::Rect {
                                c1: anchor.col.min(cursor.col),
                                r1: anchor.row.min(cursor.row),
                                c2: anchor.col.max(cursor.col),
                                r2: anchor.row.max(cursor.row),
                            }
                        }
                    };
                    ctx.wb.clear_region(spec);
                    ctx.selection = None;
                    return ModeAction::Nothing;
                }
                // 同快捷键再次按退出选中（仅 Row/Column）
                (KeyCode::Left | KeyCode::Right, Selection::Row(_))
                    if key
                        .modifiers
                        .contains(KeyModifiers::CONTROL | KeyModifiers::SHIFT) =>
                {
                    ctx.selection = None;
                    return ModeAction::Nothing;
                }
                (KeyCode::Up | KeyCode::Down, Selection::Column(_))
                    if key
                        .modifiers
                        .contains(KeyModifiers::CONTROL | KeyModifiers::SHIFT) =>
                {
                    ctx.selection = None;
                    return ModeAction::Nothing;
                }
                // 方向键退出选中并执行移动
                _ if Self::parse_direction(key).is_some() => {
                    let nav = Self::parse_direction(key).unwrap();
                    ctx.selection = None;
                    Self::apply_direction(ctx, nav);
                    return ModeAction::Nothing;
                }
                _ => return ModeAction::Nothing,
            }
        }

        // --- 非选中模式：Shift+方向键创建 Range 选区 ---
        if let Some(nav) = Self::parse_shift_direction(key) {
            let anchor = ctx.cursor;
            Self::apply_direction(ctx, nav);
            ctx.selection = Some(Selection::Range {
                anchor,
                cursor: ctx.cursor,
            });
            return ModeAction::Nothing;
        }

        // --- 非选中模式：方向键 ---
        if let Some(nav) = Self::parse_direction(key) {
            Self::apply_direction(ctx, nav);
            return ModeAction::Nothing;
        }

        // --- 非选中模式：选区快捷键（Ctrl+Shift+方向键） ---
        match key.code {
            KeyCode::Left | KeyCode::Right
                if key
                    .modifiers
                    .contains(KeyModifiers::CONTROL | KeyModifiers::SHIFT) =>
            {
                ctx.selection = Some(Selection::Row(ctx.cursor.row));
                return ModeAction::Nothing;
            }
            KeyCode::Up | KeyCode::Down
                if key
                    .modifiers
                    .contains(KeyModifiers::CONTROL | KeyModifiers::SHIFT) =>
            {
                ctx.selection = Some(Selection::Column(ctx.cursor.col));
                return ModeAction::Nothing;
            }
            _ => {}
        }

        // --- 原有逻辑 ---
        match key.code {
            KeyCode::Enter => ModeAction::SwitchToEdit { initial_char: None },
            KeyCode::Char(c) => ModeAction::SwitchToEdit {
                initial_char: Some(c),
            },
            KeyCode::Backspace | KeyCode::Delete => {
                ctx.wb.set_cell(
                    ctx.cursor,
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
                ctx.cursor.display(),
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

    /// 仅捕获 Shift+方向键（不含 Ctrl），用于框选。
    fn parse_shift_direction(key: KeyEvent) -> Option<NavigationKey> {
        if !key.modifiers.contains(KeyModifiers::SHIFT)
            || key.modifiers.contains(KeyModifiers::CONTROL)
        {
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

    fn apply_direction(ctx: &mut TableContext, nav: NavigationKey) {
        match nav {
            NavigationKey::Up if ctx.cursor.row > 0 => ctx.cursor.row -= 1,
            NavigationKey::Down if ctx.cursor.row + 1 < ctx.wb.rows => ctx.cursor.row += 1,
            NavigationKey::Left if ctx.cursor.col > 0 => ctx.cursor.col -= 1,
            NavigationKey::Right if ctx.cursor.col + 1 < ctx.wb.columns => ctx.cursor.col += 1,
            _ => {}
        }
        ctx.scroll_into_view();
    }
}
