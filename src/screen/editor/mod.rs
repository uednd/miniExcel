mod context;
mod delete;
mod edit;
mod menu;
mod mode;
mod navigation;
mod session;
mod viewport;

use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{Frame, layout::Rect, style::Style, widgets::Block};

use crate::{
    model::document::WorkbookDocument,
    theme::Theme,
    widget::table::{TableGrid, TableGridConfig, layout::GridMetrics},
};

pub use self::mode::{ModeResult, Selection};
pub use self::viewport::Viewport;

use self::session::EditorSession;

use super::{EventResult, FrameState, Screen, ScreenCommand};

pub struct TableScreen {
    session: EditorSession,
    grid_metrics: GridMetrics,
}

impl TableScreen {
    pub fn new(theme: Theme, document: WorkbookDocument) -> Self {
        Self {
            session: EditorSession::new(theme, document),
            grid_metrics: GridMetrics::new(8, 4, 2),
        }
    }
}

impl Screen for TableScreen {
    fn pre_render(&mut self, state: FrameState) {
        self.session.pre_render(state);
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let table_area = self.session.render_mode(frame, area);

        let table_block = Block::default()
            .borders(ratatui::widgets::Borders::ALL)
            .border_style(Style::default().fg(self.session.theme().accent))
            .title(format!(" {} ", self.session.workbook_name()));

        let inner = table_block.inner(table_area);
        frame.render_widget(table_block, table_area);

        let layout = self.grid_metrics.layout(inner);
        let cap = layout.visible_capacity();
        self.session.update_visible_capacity(cap.rows, cap.cols);

        let parts = self.session.table_parts();
        TableGrid::render(
            frame,
            inner,
            TableGridConfig {
                wb: parts.workbook,
                viewport: parts.viewport,
                layout,
                theme: parts.theme,
                blink_visible: parts.blink_visible,
                edit_buffer: parts.edit_buffer,
                selection: parts.selection,
                copied_region: parts.copied_region,
            },
        );
    }

    fn handle_key(&mut self, key: KeyEvent) -> EventResult<ScreenCommand> {
        self.session.handle_key(key)
    }

    fn handle_scroll(&mut self, event: MouseEvent) -> EventResult<ScreenCommand> {
        self.session.handle_scroll(event)
    }

    fn footer_hint(&self) -> Option<ratatui::text::Line<'static>> {
        self.session.footer_hint()
    }

    fn footer_status(&self) -> Option<ratatui::text::Line<'static>> {
        self.session.footer_status()
    }
}
