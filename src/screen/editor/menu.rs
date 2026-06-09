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
    mode::{FooterLine, Mode, ModeAction, ModeKind},
    navigation::NavigationMode,
};

const MENU_WIDTH: u16 = 20;

pub struct MenuMode {
    list: SelectableList,
}

impl MenuMode {
    pub fn new() -> Self {
        let items = vec![
            SelectableItem::new("保存", |ctx: &mut TableContext| {
                ctx.save();
                ModeAction::SwitchMode(Box::new(NavigationMode))
            }),
            SelectableItem::new("保存并退出", |ctx: &mut TableContext| {
                ctx.save();
                ctx.go_home();
                ModeAction::Handled
            }),
            SelectableItem::new("返回首页", |ctx: &mut TableContext| {
                ctx.go_home();
                ModeAction::Handled
            }),
        ];
        Self {
            list: SelectableList::new(items),
        }
    }
}

impl Mode for MenuMode {
    fn kind(&self) -> ModeKind {
        ModeKind::Menu
    }

    fn handle_key(&mut self, ctx: &mut TableContext, key: KeyEvent) -> ModeAction {
        match key.code {
            KeyCode::Up => {
                self.list.handle_up();
                ModeAction::Handled
            }
            KeyCode::Down => {
                self.list.handle_down();
                ModeAction::Handled
            }
            KeyCode::Enter => {
                if let Some(action) = self.list.handle_enter(ctx) {
                    action
                } else {
                    ModeAction::Handled
                }
            }
            _ => ModeAction::Handled,
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect, ctx: &TableContext) -> Rect {
        let [table_area, menu_area] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Length(MENU_WIDTH)]).areas(area);

        let menu_block = Block::default().style(Style::default().bg(ctx.theme.surface_alt));
        let inner = menu_block.inner(menu_area);
        frame.render_widget(menu_block, menu_area);
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
                Span::styled("Ctrl+P", Style::default().fg(ctx.theme.accent)),
                Span::styled(" 关闭", Style::default().fg(ctx.theme.text_dim)),
            ])),
            status: None,
        }
    }
}
