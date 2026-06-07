mod context;
mod edit;
mod menu;
mod mode;
mod navigation;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use ratatui::{Frame, layout::Rect, style::Style, widgets::Block};

use crate::{
    model::workbook::Workbook,
    theme::Theme,
    widget::table::{TableGrid, TableGridConfig},
};

use self::{
    context::TableContext,
    edit::EditMode,
    menu::MenuMode,
    mode::{Mode, ModeAction, ModeKind},
    navigation::NavigationMode,
};

use super::{Screen, ScreenCommand};

pub struct TableScreen {
    ctx: TableContext,
    mode: Box<dyn Mode>,
}

impl TableScreen {
    pub fn new(theme: Theme, path: String) -> Self {
        let wb = Workbook::load(&path).unwrap_or_else(|_| {
            let name = std::path::Path::new(&path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("untitled")
                .to_string();
            Workbook::new(name, 26, 100)
        });

        let ctx = TableContext {
            theme,
            path,
            wb,
            cursor_row: 0,
            cursor_col: 0,
            scroll_row: 0,
            scroll_col: 0,
            visible_rows: std::cell::Cell::new(0),
            visible_cols: std::cell::Cell::new(0),
        };

        Self {
            ctx,
            mode: Box::new(NavigationMode),
        }
    }
}

impl Screen for TableScreen {
    fn render(&self, frame: &mut Frame, area: Rect) {
        let table_area = self.mode.render_frame(frame, area, &self.ctx);

        let table_block = Block::default()
            .borders(ratatui::widgets::Borders::ALL)
            .border_style(Style::default().fg(self.ctx.theme.accent))
            .title(format!(" {} ", self.ctx.wb.name));

        let inner = table_block.inner(table_area);
        frame.render_widget(table_block, table_area);

        let edit_buffer = self.mode.edit_buffer();
        let (visible_rows, visible_cols) = TableGrid::render(
            frame,
            inner,
            TableGridConfig {
                wb: &self.ctx.wb,
                scroll_col: self.ctx.scroll_col,
                scroll_row: self.ctx.scroll_row,
                cursor_row: self.ctx.cursor_row,
                cursor_col: self.ctx.cursor_col,
                theme: self.ctx.theme,
                edit_buffer,
            },
        );
        self.ctx.visible_rows.set(visible_rows);
        self.ctx.visible_cols.set(visible_cols);
    }

    fn handle_key(&mut self, key: KeyEvent) -> Option<ScreenCommand> {
        if key.code == KeyCode::Char('s')
            && key.modifiers.contains(KeyModifiers::CONTROL)
            && self.mode.kind() != ModeKind::Menu
        {
            self.ctx.save();
            return Some(ScreenCommand::Stay);
        }

        if key.code == KeyCode::Char('p') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.mode = match self.mode.kind() {
                ModeKind::Menu => Box::new(NavigationMode),
                _ => Box::new(MenuMode::new()),
            };
            return Some(ScreenCommand::Stay);
        }

        match self.mode.handle_key(&mut self.ctx, key) {
            ModeAction::Nothing => Some(ScreenCommand::Stay),
            ModeAction::SwitchToEdit { initial_char } => {
                let existing = self
                    .ctx
                    .wb
                    .get_cell((self.ctx.cursor_col, self.ctx.cursor_row))
                    .map(|c| c.raw.clone())
                    .unwrap_or_default();
                self.mode = Box::new(EditMode::new(existing, initial_char));
                Some(ScreenCommand::Stay)
            }
            ModeAction::SwitchToNavigation => {
                self.mode = Box::new(NavigationMode);
                Some(ScreenCommand::Stay)
            }
            ModeAction::ScreenCommand(cmd) => Some(cmd),
        }
    }

    fn handle_scroll(&mut self, event: MouseEvent) -> Option<ScreenCommand> {
        let visible_rows = self.ctx.visible_rows.get();
        let visible_cols = self.ctx.visible_cols.get();
        let max_scroll_row = self.ctx.wb.rows.saturating_sub(visible_rows);
        let max_scroll_col = self.ctx.wb.columns.saturating_sub(visible_cols);
        match event.kind {
            MouseEventKind::ScrollUp => {
                self.ctx.scroll_row = self.ctx.scroll_row.saturating_sub(3);
                Some(ScreenCommand::Stay)
            }
            MouseEventKind::ScrollDown => {
                self.ctx.scroll_row = (self.ctx.scroll_row + 3).min(max_scroll_row);
                Some(ScreenCommand::Stay)
            }
            MouseEventKind::ScrollLeft => {
                self.ctx.scroll_col = self.ctx.scroll_col.saturating_sub(1);
                Some(ScreenCommand::Stay)
            }
            MouseEventKind::ScrollRight => {
                self.ctx.scroll_col = (self.ctx.scroll_col + 1).min(max_scroll_col);
                Some(ScreenCommand::Stay)
            }
            _ => None,
        }
    }

    fn footer_hint(&self) -> Option<ratatui::text::Line<'static>> {
        self.mode.footer_hint(&self.ctx)
    }

    fn footer_status(&self) -> Option<ratatui::text::Line<'static>> {
        self.mode.footer_status(&self.ctx)
    }
}
