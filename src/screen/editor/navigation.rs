use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{Frame, layout::Rect, style::Style, text::Line};

use crate::model::workbook::ClearSpec;

use super::{
    context::TableContext,
    edit::EditMode,
    mode::{FooterLine, Mode, ModeAction, ModeKind, Selection},
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
                    cursor: ctx.viewport.cursor(),
                });
                return ModeAction::Handled;
            }

            match (key.code, sel) {
                // Esc 退出选中
                (KeyCode::Esc, _) => {
                    ctx.selection = None;
                    return ModeAction::Handled;
                }
                // Delete/Backspace: 清空选中内容
                (KeyCode::Delete | KeyCode::Backspace, _) => {
                    let spec = match *sel {
                        Selection::Row(r) => ClearSpec::Row(r),
                        Selection::Column(c) => ClearSpec::Column(c),
                        Selection::Range { anchor, cursor } => ClearSpec::Rect {
                            c1: anchor.col.min(cursor.col),
                            r1: anchor.row.min(cursor.row),
                            c2: anchor.col.max(cursor.col),
                            r2: anchor.row.max(cursor.row),
                        },
                    };
                    ctx.wb.clear_region(spec);
                    ctx.selection = None;
                    return ModeAction::Handled;
                }
                // 同快捷键再次按退出选中（仅 Row/Column）
                (KeyCode::Left | KeyCode::Right, Selection::Row(_))
                    if key
                        .modifiers
                        .contains(KeyModifiers::CONTROL | KeyModifiers::SHIFT) =>
                {
                    ctx.selection = None;
                    return ModeAction::Handled;
                }
                (KeyCode::Up | KeyCode::Down, Selection::Column(_))
                    if key
                        .modifiers
                        .contains(KeyModifiers::CONTROL | KeyModifiers::SHIFT) =>
                {
                    ctx.selection = None;
                    return ModeAction::Handled;
                }
                // 方向键退出选中并执行移动
                _ if Self::parse_direction(key).is_some() => {
                    let nav = Self::parse_direction(key).unwrap();
                    ctx.selection = None;
                    Self::apply_direction(ctx, nav);
                    return ModeAction::Handled;
                }
                _ => return ModeAction::Handled,
            }
        }

        // --- 非选中模式：Shift+方向键创建 Range 选区 ---
        if let Some(nav) = Self::parse_shift_direction(key) {
            let anchor = ctx.viewport.cursor();
            Self::apply_direction(ctx, nav);
            ctx.selection = Some(Selection::Range {
                anchor,
                cursor: ctx.viewport.cursor(),
            });
            return ModeAction::Handled;
        }

        // --- 非选中模式：方向键 ---
        if let Some(nav) = Self::parse_direction(key) {
            Self::apply_direction(ctx, nav);
            return ModeAction::Handled;
        }

        // --- 非选中模式：选区快捷键（Ctrl+Shift+方向键） ---
        match key.code {
            KeyCode::Left | KeyCode::Right
                if key
                    .modifiers
                    .contains(KeyModifiers::CONTROL | KeyModifiers::SHIFT) =>
            {
                ctx.selection = Some(Selection::Row(ctx.viewport.cursor_row()));
                return ModeAction::Handled;
            }
            KeyCode::Up | KeyCode::Down
                if key
                    .modifiers
                    .contains(KeyModifiers::CONTROL | KeyModifiers::SHIFT) =>
            {
                ctx.selection = Some(Selection::Column(ctx.viewport.cursor_col()));
                return ModeAction::Handled;
            }
            _ => {}
        }

        // --- 原有逻辑 ---
        match key.code {
            KeyCode::Enter => {
                let existing = ctx
                    .wb
                    .get_cell(ctx.viewport.cursor())
                    .map(|c| c.raw.clone())
                    .unwrap_or_default();
                ModeAction::SwitchMode(Box::new(EditMode::new(existing, None)))
            }
            KeyCode::Char(c) => {
                let existing = ctx
                    .wb
                    .get_cell(ctx.viewport.cursor())
                    .map(|c| c.raw.clone())
                    .unwrap_or_default();
                ModeAction::SwitchMode(Box::new(EditMode::new(existing, Some(c))))
            }
            KeyCode::Backspace | KeyCode::Delete => {
                ctx.wb.set_cell(
                    ctx.viewport.cursor(),
                    String::new(),
                    crate::model::cell::CellValue::Empty,
                );
                ModeAction::Handled
            }
            _ => ModeAction::Handled,
        }
    }

    fn render(&self, _frame: &mut Frame, area: Rect, _ctx: &TableContext) -> Rect {
        area
    }

    fn footer(&self, ctx: &TableContext) -> FooterLine {
        use ratatui::text::Span;
        FooterLine {
            hint: Some(Line::from(vec![
                Span::styled("Ctrl+S", Style::default().fg(ctx.theme.accent)),
                Span::styled(" 保存", Style::default().fg(ctx.theme.text_dim)),
                Span::styled("  ", Style::default().fg(ctx.theme.text_dim)),
                Span::styled("Ctrl+P", Style::default().fg(ctx.theme.accent)),
                Span::styled(" 菜单", Style::default().fg(ctx.theme.text_dim)),
                Span::styled("  ", Style::default().fg(ctx.theme.text_dim)),
                Span::styled("Enter", Style::default().fg(ctx.theme.accent)),
                Span::styled(" 编辑", Style::default().fg(ctx.theme.text_dim)),
            ])),
            status: Some(Line::from(vec![
                Span::styled("[", Style::default().fg(ctx.theme.text_dim)),
                Span::styled(
                    ctx.viewport.cursor().display(),
                    Style::default().fg(ctx.theme.accent),
                ),
                Span::styled(", 导航模式", Style::default().fg(ctx.theme.text_dim)),
                Span::styled("]", Style::default().fg(ctx.theme.text_dim)),
            ])),
        }
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
            NavigationKey::Up => ctx.viewport.move_up(),
            NavigationKey::Down => ctx.viewport.move_down(ctx.wb.rows),
            NavigationKey::Left => ctx.viewport.move_left(),
            NavigationKey::Right => ctx.viewport.move_right(ctx.wb.columns),
        }
    }
}
