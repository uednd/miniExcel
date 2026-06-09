use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::Style,
    text::Line,
    widgets::Block,
};

use crate::screen::EventResult;
use crate::widget::selectable_list::{SelectableItem, SelectableList};

use super::{
    context::TableContext,
    mode::{FooterLine, Mode, ModeCommand, ModeKind, ModeResult},
    navigation::NavigationMode,
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
                ctx.delete_current_row();
                EventResult::Command(ModeCommand::SwitchMode(Box::new(NavigationMode)))
            }),
            SelectableItem::new("删除整列", |ctx: &mut TableContext| {
                ctx.delete_current_column();
                EventResult::Command(ModeCommand::SwitchMode(Box::new(NavigationMode)))
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

    fn handle_key(&mut self, ctx: &mut TableContext, key: KeyEvent) -> ModeResult {
        match key.code {
            KeyCode::Up => {
                self.list.handle_up();
                EventResult::Handled
            }
            KeyCode::Down => {
                self.list.handle_down();
                EventResult::Handled
            }
            KeyCode::Enter => {
                if let Some(action) = self.list.handle_enter(ctx) {
                    action
                } else {
                    EventResult::Handled
                }
            }
            KeyCode::Esc => EventResult::Command(ModeCommand::SwitchMode(Box::new(NavigationMode))),
            _ => EventResult::Handled,
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect, ctx: &TableContext) -> Rect {
        let [table_area, panel_area] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Length(PANEL_WIDTH)]).areas(area);

        let panel_block = Block::default()
            .style(Style::default().bg(ctx.theme.surface_alt))
            .title(Line::styled("删除", Style::default().fg(ctx.theme.accent)));
        let inner = panel_block.inner(panel_area);
        frame.render_widget(panel_block, panel_area);
        self.list.render(frame, inner, ctx.theme);

        table_area
    }

    fn footer(&self, ctx: &TableContext) -> FooterLine {
        use ratatui::text::Span;
        FooterLine {
            hint: Some(Line::from(vec![
                Span::styled("↑ / ↓", Style::default().fg(ctx.theme.accent)),
                Span::styled(" 选择", Style::default().fg(ctx.theme.text_dim)),
                Span::styled("  ", Style::default().fg(ctx.theme.text_dim)),
                Span::styled("Enter", Style::default().fg(ctx.theme.accent)),
                Span::styled(" 确认", Style::default().fg(ctx.theme.text_dim)),
                Span::styled("  ", Style::default().fg(ctx.theme.text_dim)),
                Span::styled("Esc", Style::default().fg(ctx.theme.accent)),
                Span::styled(" 取消", Style::default().fg(ctx.theme.text_dim)),
            ])),
            status: None,
        }
    }
}
