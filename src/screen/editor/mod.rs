mod delete;
mod edit;
mod menu;
mod mode;
mod navigation;
mod session;
mod state;
mod viewport;
mod workbook_controller;

use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{Frame, layout::Rect, style::Style, widgets::Block};

use crate::{
    model::{cell::CellValue, document::WorkbookDocument},
    theme::Theme,
    widget::table::{
        TableGrid, TableGridConfig,
        layout::GridMetrics,
        view::{GridScroll, GridSelection},
    },
};

pub use self::mode::{ModeResult, Selection};

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
        let vp = parts.viewport;
        let wb = parts.workbook;

        let scroll = GridScroll {
            scroll_row: vp.scroll_row(),
            scroll_col: vp.scroll_col(),
            visible_rows: vp.visible_rows(),
            visible_cols: vp.visible_cols(),
            cursor: vp.cursor(),
            total_rows: wb.rows,
            total_cols: wb.columns,
        };

        let map_selection = |sel: &Selection| match *sel {
            Selection::Row(r) => GridSelection::Row(r),
            Selection::Column(c) => GridSelection::Column(c),
            Selection::Range { anchor, cursor } => {
                let (r1, r2, c1, c2) = Selection::normalized(anchor, cursor);
                GridSelection::Range {
                    min_row: r1,
                    max_row: r2,
                    min_col: c1,
                    max_col: c2,
                }
            }
        };

        let cell_text_fn = |col: usize, row: usize| -> String {
            if let Some(cell) = wb.get_cell(crate::model::cell::CellAddress { row, col }) {
                match &cell.value {
                    CellValue::Number(n) => format_number(*n),
                    CellValue::Text(t) => t.clone(),
                    CellValue::Empty => String::new(),
                    CellValue::Error(e) => e.display().to_string(),
                }
            } else {
                String::new()
            }
        };

        TableGrid::render(
            frame,
            inner,
            TableGridConfig {
                scroll,
                layout,
                theme: parts.theme,
                blink_visible: parts.blink_visible,
                edit_buffer: parts.edit_buffer,
                selection: parts.selection.map(map_selection),
                copied_region: parts.copied_region.map(map_selection),
                cell_text: &cell_text_fn,
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

fn format_number(n: f64) -> String {
    if n == 0.0 {
        return "0".to_string();
    }
    let abs = n.abs();
    if !(1e-6..1e12).contains(&abs) {
        return format!("{:.2e}", n);
    }
    let s = format!("{:.10}", n);
    let s = s.trim_end_matches('0');
    s.trim_end_matches('.').to_string()
}
