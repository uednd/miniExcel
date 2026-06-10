pub mod draw;
pub mod layout;
pub mod primitives;
pub mod view;

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::{Cell, Row, Table},
};

use crate::{model::cell::col_name, theme::Theme};

use self::{
    layout::GridLayout,
    primitives::{BorderStroke, CellRect, RegionRect},
    view::{GridScroll, GridSelection},
};

pub struct TableGrid;

/// 表格渲染输入。
///
/// `layout` 由调用方根据目标区域构造，内部携带坐标映射逻辑。
pub struct TableGridConfig<'a> {
    pub scroll: GridScroll,
    pub layout: GridLayout,
    pub theme: Theme,
    pub blink_visible: bool,
    pub edit_buffer: Option<&'a str>,
    pub selection: Option<GridSelection>,
    pub copied_region: Option<GridSelection>,
    pub cell_text: &'a dyn Fn(usize, usize) -> String,
}

impl TableGrid {
    /// 渲染表格网格、表头、单元格内容、光标和选区。
    pub fn render(frame: &mut Frame, area: Rect, cfg: TableGridConfig) {
        let sc = cfg.scroll;

        if sc.visible_rows == 0 || sc.visible_cols == 0 {
            return;
        }

        let header_style = Style::default().fg(cfg.theme.text_dim);
        let selected_header_style = Style::default()
            .fg(cfg.theme.accent)
            .bg(cfg.theme.table_header_highlight_bg)
            .add_modifier(Modifier::BOLD);
        let cell_style = Style::default().fg(cfg.theme.text_dim);
        let grid_style = Style::default().fg(cfg.theme.grid);

        let col_end = (sc.scroll_col + sc.visible_cols).min(sc.total_cols);
        let rendered_cols = col_end - sc.scroll_col;
        let widths: Vec<Constraint> = cfg
            .layout
            .table_column_widths(rendered_cols)
            .into_iter()
            .map(Constraint::Length)
            .collect();

        // 表头行
        let mut header_cells = Vec::with_capacity(col_end.saturating_sub(sc.scroll_col) + 1);
        header_cells.push(Cell::from("").style(header_style));
        for ci in sc.scroll_col..col_end {
            let style = if ci == sc.cursor.col {
                selected_header_style
            } else {
                header_style
            };
            header_cells.push(
                Cell::from(Line::from(col_name(ci)).alignment(Alignment::Center)).style(style),
            );
        }

        let row_end = (sc.scroll_row + sc.visible_rows).min(sc.total_rows);
        let rendered_rows = row_end - sc.scroll_row;

        // 数据行
        let mut rows: Vec<Row> = Vec::with_capacity(row_end.saturating_sub(sc.scroll_row));
        for ri in sc.scroll_row..row_end {
            let label_style = if ri == sc.cursor.row {
                selected_header_style
            } else {
                header_style
            };
            let mut content_cells = vec![
                Cell::from(Line::from((ri + 1).to_string()).alignment(Alignment::Right))
                    .style(label_style),
            ];
            for ci in sc.scroll_col..col_end {
                let text = if ci == sc.cursor.col && ri == sc.cursor.row {
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
                        (cfg.cell_text)(ci, ri)
                    }
                } else {
                    (cfg.cell_text)(ci, ri)
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
                selection_to_region_rect(&cfg.layout, &sc, &sel, rendered_rows, rendered_cols)
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
                selection_to_region_rect(&cfg.layout, &sc, &region, rendered_rows, rendered_cols)
        {
            draw::draw_region_border(
                buffer,
                rect,
                Style::default().fg(cfg.theme.accent),
                BorderStroke::Dashed,
            );
        }

        // 光标单元格高亮边框
        if let Some(cell_rect) = cursor_cell_rect(&cfg.layout, &sc, cfg.selection) {
            draw::draw_cell_border(buffer, cell_rect, Style::default().fg(cfg.theme.accent));
        }
    }
}

fn cursor_cell_rect(
    layout: &GridLayout,
    scroll: &GridScroll,
    selection: Option<GridSelection>,
) -> Option<CellRect> {
    if let Some(sel) = selection
        && sel.contains(scroll.cursor)
    {
        return None;
    }
    let vis_row = scroll.cursor.row.checked_sub(scroll.scroll_row)?;
    if vis_row >= scroll.visible_rows {
        return None;
    }
    let vis_col = scroll.cursor.col.checked_sub(scroll.scroll_col)?;
    if vis_col >= scroll.visible_cols {
        return None;
    }
    layout.cell_rect(vis_row, vis_col)
}

fn selection_to_region_rect(
    layout: &GridLayout,
    scroll: &GridScroll,
    selection: &GridSelection,
    rendered_rows: usize,
    rendered_cols: usize,
) -> Option<RegionRect> {
    match *selection {
        GridSelection::Row(r) => {
            let vis_row = r.checked_sub(scroll.scroll_row)?;
            if vis_row >= rendered_rows {
                return None;
            }
            layout.row_region_rect(vis_row, rendered_cols)
        }
        GridSelection::Column(c) => {
            let vis_col = c.checked_sub(scroll.scroll_col)?;
            if vis_col >= rendered_cols {
                return None;
            }
            layout.col_region_rect(vis_col, rendered_rows)
        }
        GridSelection::Range {
            min_row: r1,
            max_row: r2,
            min_col: c1,
            max_col: c2,
        } => {
            if r1 >= scroll.scroll_row + rendered_rows || r2 < scroll.scroll_row {
                return None;
            }
            if c1 >= scroll.scroll_col + rendered_cols || c2 < scroll.scroll_col {
                return None;
            }
            let vis_r1 = r1.max(scroll.scroll_row).saturating_sub(scroll.scroll_row);
            let vis_r2 = r2
                .min(scroll.scroll_row + rendered_rows.saturating_sub(1))
                .saturating_sub(scroll.scroll_row);
            let vis_c1 = c1.max(scroll.scroll_col).saturating_sub(scroll.scroll_col);
            let vis_c2 = c2
                .min(scroll.scroll_col + rendered_cols.saturating_sub(1))
                .saturating_sub(scroll.scroll_col);
            layout.region_rect(vis_r1..=vis_r2, vis_c1..=vis_c2)
        }
    }
}
