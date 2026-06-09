mod context;
mod delete;
mod edit;
mod host;
mod menu;
mod mode;
mod navigation;
mod viewport;

use crossterm::event::{KeyEvent, MouseEvent, MouseEventKind};
use ratatui::{Frame, layout::Rect, style::Style, widgets::Block};

use crate::{
    model::{
        limits::{MAX_COLUMNS, MAX_ROWS},
        workbook::Workbook,
    },
    theme::Theme,
    widget::table::{COL_WIDTH, ROW_NUM_WIDTH, TableGrid, TableGridConfig},
};

pub use self::context::TableContext;
pub use self::mode::{ModeAction, Selection};
pub use self::viewport::Viewport;

use self::{host::ModeHost, navigation::NavigationMode};

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
            viewport: Viewport::new(),
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
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let table_area = self.host.render(frame, area, &self.ctx);

        let table_block = Block::default()
            .borders(ratatui::widgets::Borders::ALL)
            .border_style(Style::default().fg(self.ctx.theme.accent))
            .title(format!(" {} ", self.ctx.wb.name));

        let inner = table_block.inner(table_area);
        frame.render_widget(table_block, table_area);

        let visible_rows = (inner.height.saturating_sub(2) / 2) as usize;
        let visible_cols = ((inner.width.saturating_sub(ROW_NUM_WIDTH)) / (COL_WIDTH + 1)) as usize;
        self.ctx.viewport.update_visible(visible_rows, visible_cols);

        let edit_buffer = self.host.edit_buffer();
        let selection = self.ctx.selection.as_ref();
        TableGrid::render(
            frame,
            inner,
            TableGridConfig {
                wb: &self.ctx.wb,
                viewport: &self.ctx.viewport,
                theme: self.ctx.theme,
                edit_buffer,
                selection,
            },
        );
    }

    fn handle_key(&mut self, key: KeyEvent) -> Option<ScreenCommand> {
        self.host.handle_key(&mut self.ctx, key)
    }

    fn handle_scroll(&mut self, event: MouseEvent) -> Option<ScreenCommand> {
        match event.kind {
            MouseEventKind::ScrollUp => {
                self.ctx.viewport.scroll_up(3);
                Some(ScreenCommand::Stay)
            }
            MouseEventKind::ScrollDown => {
                self.ctx.viewport.scroll_down(3, self.ctx.wb.rows);
                Some(ScreenCommand::Stay)
            }
            MouseEventKind::ScrollLeft => {
                self.ctx.viewport.scroll_left(1);
                Some(ScreenCommand::Stay)
            }
            MouseEventKind::ScrollRight => {
                self.ctx.viewport.scroll_right(1, self.ctx.wb.columns);
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
