use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::Style,
    text::Line,
    widgets::Block,
};

use crate::widget::selectable_list::{SelectableItem, SelectableList};

use super::{
    context::TableContext,
    mode::{Mode, ModeAction, ModeKind},
};

const PANEL_WIDTH: u16 = 20;

/// 删除面板：通过 SelectableList 提供删除整行/整列操作。
/// 由 Ctrl+D 触发，Enter 执行选中操作，Esc 关闭。
pub struct DeleteMode {
    list: SelectableList,
}

impl DeleteMode {
    pub fn new() -> Self {
        let items = vec![
            SelectableItem::new("删除整行", |ctx: &mut TableContext| {
                ctx.wb.delete_row(ctx.cursor_row);
                if ctx.cursor_row >= ctx.wb.rows {
                    ctx.cursor_row = ctx.wb.rows.saturating_sub(1);
                }
                ctx.scroll_into_view();
                ModeAction::SwitchToNavigation
            }),
            SelectableItem::new("删除整列", |ctx: &mut TableContext| {
                ctx.wb.delete_column(ctx.cursor_col);
                if ctx.cursor_col >= ctx.wb.columns {
                    ctx.cursor_col = ctx.wb.columns.saturating_sub(1);
                }
                ctx.scroll_into_view();
                ModeAction::SwitchToNavigation
            }),
        ];
        Self {
            list: SelectableList::new(items),
        }
    }
}

impl Mode for DeleteMode {
    fn kind(&self) -> ModeKind {
        ModeKind::Delete
    }

    fn handle_key(&mut self, ctx: &mut TableContext, key: KeyEvent) -> ModeAction {
        match key.code {
            KeyCode::Up => {
                self.list.handle_up();
                ModeAction::Nothing
            }
            KeyCode::Down => {
                self.list.handle_down();
                ModeAction::Nothing
            }
            KeyCode::Enter => {
                if let Some(action) = self.list.handle_enter(ctx) {
                    action
                } else {
                    ModeAction::Nothing
                }
            }
            KeyCode::Esc => ModeAction::SwitchToNavigation,
            _ => ModeAction::Nothing,
        }
    }

    fn render_frame(&self, frame: &mut Frame, area: Rect, ctx: &TableContext) -> Rect {
        let [table_area, panel_area] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Length(PANEL_WIDTH)]).areas(area);

        let panel_block =
            Block::default().style(Style::default().bg(ctx.theme.surface_alt)).title(Line::styled("删除", Style::default().fg(ctx.theme.accent)));
        let inner = panel_block.inner(panel_area);
        frame.render_widget(panel_block, panel_area);
        self.list.render(frame, inner, ctx.theme);

        table_area
    }

    fn footer_hint(&self, ctx: &TableContext) -> Option<Line<'static>> {
        use ratatui::text::Span;
        Some(Line::from(vec![
            Span::styled("↑ / ↓", Style::default().fg(ctx.theme.accent)),
            Span::styled(" 选择", Style::default().fg(ctx.theme.text_dim)),
            Span::styled("  ", Style::default().fg(ctx.theme.text_dim)),
            Span::styled("Enter", Style::default().fg(ctx.theme.accent)),
            Span::styled(" 确认", Style::default().fg(ctx.theme.text_dim)),
            Span::styled("  ", Style::default().fg(ctx.theme.text_dim)),
            Span::styled("Esc", Style::default().fg(ctx.theme.accent)),
            Span::styled(" 取消", Style::default().fg(ctx.theme.text_dim)),
        ]))
    }
}
