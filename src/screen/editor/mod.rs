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
use std::path::PathBuf;

use crate::{
    model::{
        limits::{MAX_COLUMNS, MAX_ROWS},
        workbook::Workbook,
    },
    theme::Theme,
    widget::table::{TableGrid, TableGridConfig, layout::GridMetrics},
};

pub use self::context::TableContext;
pub use self::mode::{ModeResult, Selection};
pub use self::viewport::Viewport;

use self::{host::ModeHost, navigation::NavigationMode};

use super::{EventResult, FrameState, Screen, ScreenCommand};

pub struct TableScreen {
    ctx: TableContext,
    host: ModeHost,
    grid_metrics: GridMetrics,
}

impl TableScreen {
    pub fn new(theme: Theme, path: PathBuf) -> Self {
        let wb = Workbook::load(&path).unwrap_or_else(|_| {
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("untitled")
                .to_string();
            Workbook::new(name, MAX_COLUMNS, MAX_ROWS)
        });

        let ctx = TableContext::new(theme, path, wb);

        Self {
            ctx,
            host: ModeHost::new(Box::new(NavigationMode)),
            grid_metrics: GridMetrics::new(8, 4, 2),
        }
    }
}

impl Screen for TableScreen {
    fn pre_render(&mut self, state: FrameState) {
        self.ctx.set_blink_visible(state.blink_visible);
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let table_area = self.host.render(frame, area, &self.ctx);

        let table_block = Block::default()
            .borders(ratatui::widgets::Borders::ALL)
            .border_style(Style::default().fg(self.ctx.theme.accent))
            .title(format!(" {} ", self.ctx.workbook_name()));

        let inner = table_block.inner(table_area);
        frame.render_widget(table_block, table_area);

        let layout = self.grid_metrics.layout(inner);
        let cap = layout.visible_capacity();
        self.ctx.viewport.update_visible(cap.rows, cap.cols);

        let edit_buffer = self.host.edit_buffer();
        let selection = self.ctx.selection();
        let copied_region = self.ctx.copied_region();
        TableGrid::render(
            frame,
            inner,
            TableGridConfig {
                wb: self.ctx.workbook(),
                viewport: &self.ctx.viewport,
                layout,
                theme: self.ctx.theme,
                blink_visible: self.ctx.blink_visible(),
                edit_buffer,
                selection,
                copied_region,
            },
        );
    }

    fn handle_key(&mut self, key: KeyEvent) -> EventResult<ScreenCommand> {
        self.host.handle_key(&mut self.ctx, key)
    }

    fn handle_scroll(&mut self, event: MouseEvent) -> EventResult<ScreenCommand> {
        match event.kind {
            MouseEventKind::ScrollUp => {
                self.ctx.viewport.scroll_up(3);
                EventResult::Handled
            }
            MouseEventKind::ScrollDown => {
                self.ctx.viewport.scroll_down(3, self.ctx.row_count());
                EventResult::Handled
            }
            MouseEventKind::ScrollLeft => {
                self.ctx.viewport.scroll_left(1);
                EventResult::Handled
            }
            MouseEventKind::ScrollRight => {
                self.ctx.viewport.scroll_right(1, self.ctx.column_count());
                EventResult::Handled
            }
            _ => EventResult::Ignored,
        }
    }

    fn footer_hint(&self) -> Option<ratatui::text::Line<'static>> {
        self.host.footer(&self.ctx).hint
    }

    fn footer_status(&self) -> Option<ratatui::text::Line<'static>> {
        self.host.footer(&self.ctx).status
    }
}
