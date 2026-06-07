use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, List, ListItem, ListState},
};

use super::{
    context::TableContext,
    mode::{Mode, ModeAction, ModeKind},
};
use crate::screen::ScreenCommand;

const MENU_WIDTH: u16 = 20;

const MENU_ITEMS: [MenuItem; 3] = [MenuItem::Save, MenuItem::SaveAndExit, MenuItem::GoHome];

enum MenuItem {
    Save,
    SaveAndExit,
    GoHome,
}

impl MenuItem {
    fn label(&self) -> &'static str {
        match self {
            MenuItem::Save => "保存",
            MenuItem::SaveAndExit => "保存并退出",
            MenuItem::GoHome => "返回首页",
        }
    }

    fn handle(&self, ctx: &mut TableContext) -> ModeAction {
        match self {
            MenuItem::Save => {
                ctx.save();
                ModeAction::SwitchToNavigation
            }
            MenuItem::SaveAndExit => {
                ctx.save();
                ModeAction::ScreenCommand(ScreenCommand::GoHome)
            }
            MenuItem::GoHome => ModeAction::ScreenCommand(ScreenCommand::GoHome),
        }
    }
}

/// 侧栏面板
pub struct MenuMode {
    selected: usize,
    items: &'static [MenuItem],
}

impl MenuMode {
    pub fn new() -> Self {
        Self {
            selected: 0,
            items: &MENU_ITEMS,
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
                if self.selected > 0 {
                    self.selected -= 1;
                }
                ModeAction::Nothing
            }
            KeyCode::Down => {
                if self.selected + 1 < self.items.len() {
                    self.selected += 1;
                }
                ModeAction::Nothing
            }
            KeyCode::Enter => self
                .items
                .get(self.selected)
                .map(|item| item.handle(ctx))
                .unwrap_or(ModeAction::Nothing),
            _ => ModeAction::Nothing,
        }
    }

    fn render_frame(&self, frame: &mut Frame, area: Rect, ctx: &TableContext) -> Rect {
        let [table_area, menu_area] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Length(MENU_WIDTH)]).areas(area);

        let menu_block = Block::default().style(Style::default().bg(ctx.theme.surface_alt));
        let inner = menu_block.inner(menu_area);
        frame.render_widget(menu_block, menu_area);

        let [_, items_area, _] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(self.items.len() as u16 * 2),
            Constraint::Fill(1),
        ])
        .areas(inner);

        let menu_items: Vec<ListItem> = self
            .items
            .iter()
            .map(|item| {
                ListItem::new(Line::from(item.label())).style(Style::default().fg(ctx.theme.text_dim))
            })
            .collect();
        let menu = List::new(menu_items)
            .highlight_symbol(Span::styled("▎", Style::default().fg(ctx.theme.accent)))
            .highlight_style(Style::default().fg(ctx.theme.text).bg(ctx.theme.surface));
        let mut state = ListState::default();
        state.select(Some(self.selected));

        frame.render_stateful_widget(menu, items_area, &mut state);

        table_area
    }

    fn footer_hint(&self, ctx: &TableContext) -> Option<Line<'static>> {
        Some(Line::from(vec![
            Span::styled("↑ / ↓", Style::default().fg(ctx.theme.accent)),
            Span::styled(" 选择", Style::default().fg(ctx.theme.text_dim)),
            Span::styled("  ", Style::default().fg(ctx.theme.text_dim)),
            Span::styled("Enter", Style::default().fg(ctx.theme.accent)),
            Span::styled(" 确认", Style::default().fg(ctx.theme.text_dim)),
            Span::styled("  ", Style::default().fg(ctx.theme.text_dim)),
            Span::styled("Ctrl+P", Style::default().fg(ctx.theme.accent)),
            Span::styled(" 关闭", Style::default().fg(ctx.theme.text_dim)),
        ]))
    }
}
