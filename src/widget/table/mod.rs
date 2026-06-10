pub mod draw;
pub mod layout;
pub mod primitives;

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::{Cell, Row, Table},
};

use crate::{
    model::{
        cell::{CellAddress, CellValue, col_name},
        workbook::Workbook,
    },
    screen::editor::{Selection, Viewport},
    theme::Theme,
};

use self::{
    layout::GridLayout,
    primitives::{BorderStroke, CellRect, RegionRect},
};

pub struct TableGrid;

/// 表格渲染输入。
///
/// `layout` 由调用方根据目标区域构造，内部携带坐标映射逻辑。
pub struct TableGridConfig<'a> {
    pub wb: &'a Workbook,
    pub viewport: &'a Viewport,
    pub layout: GridLayout,
    pub theme: Theme,
    pub blink_visible: bool,
    pub edit_buffer: Option<&'a str>,
    pub selection: Option<&'a Selection>,
    pub copied_region: Option<&'a Selection>,
}

impl TableGrid {
    /// 渲染表格网格、表头、单元格内容、光标和选区。
    pub fn render(frame: &mut Frame, area: Rect, cfg: TableGridConfig) {
        let vp = cfg.viewport;
        let cursor = vp.cursor();
        let visible_rows = vp.visible_rows();
        let visible_cols = vp.visible_cols();

        if visible_rows == 0 || visible_cols == 0 {
            return;
        }

        let header_style = Style::default().fg(cfg.theme.text_dim);
        let selected_header_style = Style::default()
            .fg(cfg.theme.accent)
            .bg(cfg.theme.table_header_highlight_bg)
            .add_modifier(Modifier::BOLD);
        let cell_style = Style::default().fg(cfg.theme.text_dim);
        let grid_style = Style::default().fg(cfg.theme.grid);

        let col_end = (vp.scroll_col() + visible_cols).min(cfg.wb.columns);
        let rendered_cols = col_end - vp.scroll_col();
        let widths: Vec<Constraint> = cfg
            .layout
            .table_column_widths(rendered_cols)
            .into_iter()
            .map(Constraint::Length)
            .collect();

        // 表头行
        let mut header_cells = Vec::with_capacity(col_end.saturating_sub(vp.scroll_col()) + 1);
        header_cells.push(Cell::from("").style(header_style));
        for ci in vp.scroll_col()..col_end {
            let style = if ci == cursor.col {
                selected_header_style
            } else {
                header_style
            };
            header_cells.push(
                Cell::from(Line::from(col_name(ci)).alignment(Alignment::Center)).style(style),
            );
        }

        let row_end = (vp.scroll_row() + visible_rows).min(cfg.wb.rows);
        let rendered_rows = row_end - vp.scroll_row();

        // 数据行
        let mut rows: Vec<Row> = Vec::with_capacity(row_end.saturating_sub(vp.scroll_row()));
        for ri in vp.scroll_row()..row_end {
            let label_style = if ri == cursor.row {
                selected_header_style
            } else {
                header_style
            };
            let mut content_cells = vec![
                Cell::from(Line::from((ri + 1).to_string()).alignment(Alignment::Right))
                    .style(label_style),
            ];
            for ci in vp.scroll_col()..col_end {
                let text = if ci == cursor.col && ri == cursor.row {
                    if let Some(buffer) = cfg.edit_buffer {
                        if cfg.blink_visible {
                            if buffer.is_empty() {
                                "█".to_string()
                            } else {
                                format!("{}█", buffer)
                            }
                        } else {
                            buffer.to_string()
                        }
                    } else {
                        cell_text(cfg.wb, ci, ri)
                    }
                } else {
                    cell_text(cfg.wb, ci, ri)
                };
                content_cells.push(Cell::from(text).style(cell_style));
            }
            rows.push(Row::new(content_cells).bottom_margin(1));
        }

        let table = Table::new(rows, widths)
            .header(Row::new(header_cells).bottom_margin(1))
            .column_spacing(1)
            .highlight_symbol("");

        frame.render_widget(table, area);

        let buffer = frame.buffer_mut();
        let horizontal_ys = cfg.layout.horizontal_grid_ys(rendered_rows);
        let vertical_xs = cfg.layout.vertical_grid_xs(rendered_cols);
        draw::draw_grid(buffer, area, &horizontal_ys, &vertical_xs, grid_style);

        // 绘制选区边框（实线）
        if let Some(sel) = cfg.selection
            && let Some(region) =
                selection_to_region_rect(&cfg.layout, vp, sel, rendered_rows, rendered_cols)
        {
            draw::draw_region_border(
                buffer,
                region,
                Style::default().fg(cfg.theme.accent),
                BorderStroke::Solid,
            );
        }

        // 绘制已复制区域边框（虚线）
        if let Some(region) = cfg.copied_region
            && let Some(rect) =
                selection_to_region_rect(&cfg.layout, vp, region, rendered_rows, rendered_cols)
        {
            draw::draw_region_border(
                buffer,
                rect,
                Style::default().fg(cfg.theme.accent),
                BorderStroke::Dashed,
            );
        }

        // 光标单元格高亮边框
        if let Some(cell_rect) = cursor_cell_rect(&cfg.layout, vp, cursor, cfg.selection) {
            draw::draw_cell_border(buffer, cell_rect, Style::default().fg(cfg.theme.accent));
        }
    }
}

fn cursor_cell_rect(
    layout: &GridLayout,
    viewport: &Viewport,
    cursor: CellAddress,
    selection: Option<&Selection>,
) -> Option<CellRect> {
    if let Some(sel) = selection
        && sel.contains(cursor)
    {
        return None;
    }
    let vis_row = cursor.row.checked_sub(viewport.scroll_row())?;
    if vis_row >= viewport.visible_rows() {
        return None;
    }
    let vis_col = cursor.col.checked_sub(viewport.scroll_col())?;
    if vis_col >= viewport.visible_cols() {
        return None;
    }
    layout.cell_rect(vis_row, vis_col)
}

fn selection_to_region_rect(
    layout: &GridLayout,
    viewport: &Viewport,
    selection: &Selection,
    rendered_rows: usize,
    rendered_cols: usize,
) -> Option<RegionRect> {
    let scroll_row = viewport.scroll_row();
    let scroll_col = viewport.scroll_col();

    match *selection {
        Selection::Row(r) => {
            let vis_row = r.checked_sub(scroll_row)?;
            if vis_row >= rendered_rows {
                return None;
            }
            layout.row_region_rect(vis_row, rendered_cols)
        }
        Selection::Column(c) => {
            let vis_col = c.checked_sub(scroll_col)?;
            if vis_col >= rendered_cols {
                return None;
            }
            layout.col_region_rect(vis_col, rendered_rows)
        }
        Selection::Range { anchor, cursor } => {
            let (r1, r2, c1, c2) = Selection::normalized(anchor, cursor);
            if r1 >= scroll_row + rendered_rows || r2 < scroll_row {
                return None;
            }
            if c1 >= scroll_col + rendered_cols || c2 < scroll_col {
                return None;
            }
            let vis_r1 = r1.max(scroll_row).saturating_sub(scroll_row);
            let vis_r2 = r2
                .min(scroll_row + rendered_rows.saturating_sub(1))
                .saturating_sub(scroll_row);
            let vis_c1 = c1.max(scroll_col).saturating_sub(scroll_col);
            let vis_c2 = c2
                .min(scroll_col + rendered_cols.saturating_sub(1))
                .saturating_sub(scroll_col);
            layout.region_rect(vis_r1..=vis_r2, vis_c1..=vis_c2)
        }
    }
}

/// 从 HashMap 读取单元格并转成显示字符串
fn cell_text(wb: &Workbook, col: usize, row: usize) -> String {
    if let Some(cell) = wb.get_cell(CellAddress { row, col }) {
        match &cell.value {
            CellValue::Number(n) => format_number(*n),
            CellValue::Text(t) => t.clone(),
            CellValue::Empty => String::new(),
            CellValue::Error(e) => e.display().to_string(),
        }
    } else {
        String::new()
    }
}

fn format_number(n: f64) -> String {
    if n == 0.0 {
        return "0".to_string();
    }
    let abs = n.abs();
    if abs < 1e-6 || abs >= 1e12 {
        return format!("{:.2e}", n);
    }
    let s = format!("{:.10}", n);
    let s = s.trim_end_matches('0');
    s.trim_end_matches('.').to_string()
}
