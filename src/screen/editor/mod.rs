mod context;
mod delete;
mod edit;
mod host;
mod menu;
mod mode;
mod navigation;

use crossterm::event::{KeyEvent, MouseEvent, MouseEventKind};
use ratatui::{Frame, layout::Rect, style::Style, widgets::Block};

use crate::{
    model::{
        cell::CellAddress,
        limits::{MAX_COLUMNS, MAX_ROWS},
        workbook::Workbook,
    },
    theme::Theme,
    widget::table::{TableGrid, TableGridConfig},
};

pub use self::context::TableContext;
pub use self::mode::{ModeAction, Selection};

use self::{
    host::ModeHost,
    navigation::NavigationMode,
};

use super::{Screen, ScreenCommand};

pub struct TableScreen {
    ctx: TableContext,
    host: ModeHost,
}

impl TableScreen {
    pub fn new(theme: Theme, path: String) -> Self {
        let wb = Workbook::load(&path).unwrap_or_else(|_| {
            let name = std::path::Path::new(&path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("untitled")
                .to_string();
            Workbook::new(name, MAX_COLUMNS, MAX_ROWS)
        });

        let ctx = TableContext {
            theme,
            path,
            wb,
            cursor: CellAddress::new(0, 0),
            scroll_row: 0,
            scroll_col: 0,
            visible_rows: std::cell::Cell::new(0),
            visible_cols: std::cell::Cell::new(0),
            selection: None,
            pending_command: None,
        };

        Self {
            ctx,
            host: ModeHost::new(Box::new(NavigationMode)),
        }
    }
}

impl Screen for TableScreen {
    fn render(&self, frame: &mut Frame, area: Rect) {
        let table_area = self.host.render(frame, area, &self.ctx);

        let table_block = Block::default()
            .borders(ratatui::widgets::Borders::ALL)
            .border_style(Style::default().fg(self.ctx.theme.accent))
            .title(format!(" {} ", self.ctx.wb.name));

        let inner = table_block.inner(table_area);
        frame.render_widget(table_block, table_area);

        let edit_buffer = self.host.edit_buffer();
        let selection = self.ctx.selection.as_ref();
        let (visible_rows, visible_cols) = TableGrid::render(
            frame,
            inner,
            TableGridConfig {
                wb: &self.ctx.wb,
                scroll_col: self.ctx.scroll_col,
                scroll_row: self.ctx.scroll_row,
                cursor: self.ctx.cursor,
                theme: self.ctx.theme,
                edit_buffer,
                selection,
            },
        );
        self.ctx.visible_rows.set(visible_rows);
        self.ctx.visible_cols.set(visible_cols);
    }

    fn handle_key(&mut self, key: KeyEvent) -> Option<ScreenCommand> {
        self.host.handle_key(&mut self.ctx, key)
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
        self.host.footer(&self.ctx).hint
    }

    fn footer_status(&self) -> Option<ratatui::text::Line<'static>> {
        self.host.footer(&self.ctx).status
    }
}
