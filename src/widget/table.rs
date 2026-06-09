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
    screen::editor::Selection,
    theme::Theme,
    util::blink_visible,
};

/// 单元格列宽
pub const COL_WIDTH: u16 = 8;
/// 行号列列宽
pub const ROW_NUM_WIDTH: u16 = 4;

pub struct TableGrid;

pub struct TableGridConfig<'a> {
    pub wb: &'a Workbook,
    pub scroll_col: usize,
    pub scroll_row: usize,
    pub cursor: CellAddress,
    pub theme: Theme,
    pub edit_buffer: Option<&'a str>,
    pub selection: Option<&'a Selection>,
}

impl TableGrid {
    pub fn render(frame: &mut Frame, area: Rect, cfg: TableGridConfig) -> (usize, usize) {
        let visible_rows = area.height.saturating_sub(2) as usize / 2;
        let visible_cols = (area.width.saturating_sub(ROW_NUM_WIDTH) / (COL_WIDTH + 1)) as usize;

        if visible_rows == 0 || visible_cols == 0 {
            return (0, 0);
        }

        let header_style = Style::default().fg(cfg.theme.text_dim);
        let selected_header_style = Style::default()
            .fg(cfg.theme.accent)
            .bg(cfg.theme.table_header_highlight_bg)
            .add_modifier(Modifier::BOLD);
        let cell_style = Style::default().fg(cfg.theme.text_dim);
        let grid_style = Style::default().fg(cfg.theme.grid);

        let mut widths = vec![Constraint::Length(ROW_NUM_WIDTH)];

        // 构造每行的列宽配置
        for _ in 0..visible_cols {
            widths.push(Constraint::Length(COL_WIDTH));
        }

        let col_end = (cfg.scroll_col + visible_cols).min(cfg.wb.columns);

        // 表头行
        let mut header_cells = Vec::with_capacity(col_end.saturating_sub(cfg.scroll_col) + 1);
        header_cells.push(Cell::from("").style(header_style));
        for ci in cfg.scroll_col..col_end {
            // 光标列高亮
            let style = if ci == cfg.cursor.col {
                selected_header_style
            } else {
                header_style
            };
            header_cells.push(
                Cell::from(Line::from(col_name(ci)).alignment(Alignment::Center)).style(style),
            );
        }

        let row_end = (cfg.scroll_row + visible_rows).min(cfg.wb.rows);

        // 数据行
        let mut rows: Vec<Row> = Vec::with_capacity(row_end.saturating_sub(cfg.scroll_row));
        for ri in cfg.scroll_row..row_end {
            let label_style = if ri == cfg.cursor.row {
                selected_header_style
            } else {
                header_style
            };
            // 行号
            let mut content_cells = vec![
                Cell::from(Line::from((ri + 1).to_string()).alignment(Alignment::Right))
                    .style(label_style),
            ];
            for ci in cfg.scroll_col..col_end {
                let text = if ci == cfg.cursor.col && ri == cfg.cursor.row {
                    // 编辑模式
                    if let Some(buffer) = cfg.edit_buffer {
                        if blink_visible() {
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

        draw_grid(frame, area, visible_cols, grid_style);

        // 绘制选中行/列边框
        if let Some(sel) = cfg.selection {
            draw_selection_border(
                frame,
                area,
                sel,
                cfg.scroll_row,
                cfg.scroll_col,
                (row_end - cfg.scroll_row, col_end - cfg.scroll_col),
                Style::default().fg(cfg.theme.accent),
            );
        }

        // 绘制选中单元格高亮边框（若光标在选中行/列内则跳过，避免边框重叠）
        let overlap = match cfg.selection {
            Some(Selection::Row(r)) => *r == cfg.cursor.row,
            Some(Selection::Column(c)) => *c == cfg.cursor.col,
            Some(Selection::Range { anchor, cursor }) => {
                let c1 = anchor.col.min(cursor.col);
                let c2 = anchor.col.max(cursor.col);
                let r1 = anchor.row.min(cursor.row);
                let r2 = anchor.row.max(cursor.row);
                cfg.cursor.col >= c1 && cfg.cursor.col <= c2 && cfg.cursor.row >= r1 && cfg.cursor.row <= r2
            }
            _ => false,
        };
        if !overlap
            && cfg.cursor.row >= cfg.scroll_row
            && cfg.cursor.row < cfg.scroll_row + visible_rows
            && cfg.cursor.col >= cfg.scroll_col
            && cfg.cursor.col < cfg.scroll_col + visible_cols
        {
            draw_cell_border(
                frame,
                area,
                cfg.cursor.row - cfg.scroll_row,
                cfg.cursor.col - cfg.scroll_col,
                Style::default().fg(cfg.theme.accent),
            );
        }

        (visible_rows, visible_cols)
    }
}

/// 绘制网格线
fn draw_grid(frame: &mut Frame, area: Rect, visible_cols: usize, grid_style: Style) {
    let buffer = frame.buffer_mut();

    // 水平线
    for y in area.y + 1..area.bottom() {
        if is_grid_row(y, area.y) {
            for x in area.x..area.right() {
                buffer[(x, y)].set_symbol("─").set_style(grid_style);
            }
        }
    }

    // 垂直线
    for ci in 0..=visible_cols {
        let x = col_grid_x(area, ci);
        if x >= area.right() {
            continue;
        }
        for y in area.y..area.bottom() {
            let sym = if y == area.y {
                if ci == 0 { " " } else { "│" }
            } else if is_grid_row(y, area.y) {
                "┼"
            } else {
                "│"
            };
            buffer[(x, y)].set_symbol(sym).set_style(grid_style);
        }
    }
}

fn col_grid_x(area: Rect, col_index: usize) -> u16 {
    area.x + ROW_NUM_WIDTH + col_index as u16 * (COL_WIDTH + 1)
}

fn is_grid_row(y: u16, area_top: u16) -> bool {
    y > area_top && (y - area_top - 1).is_multiple_of(2)
}

/// 光标的表格坐标转成屏幕坐标
fn draw_cell_border(
    frame: &mut Frame,
    area: Rect,
    visible_row: usize,
    visible_col: usize,
    style: Style,
) {
    let left = col_grid_x(area, visible_col);
    let right = col_grid_x(area, visible_col + 1);
    let top = area.y + 1 + visible_row as u16 * 2;
    let content = top + 1;
    let bottom = top + 2;

    if left >= right || right >= area.right() || bottom >= area.bottom() {
        return;
    }

    let buffer = frame.buffer_mut();
    buffer[(left, top)].set_symbol("╆").set_style(style);
    buffer[(right, top)].set_symbol("╅").set_style(style);
    buffer[(left, bottom)].set_symbol("╄").set_style(style);
    buffer[(right, bottom)].set_symbol("╃").set_style(style);

    for x in left + 1..right {
        buffer[(x, top)].set_symbol("━").set_style(style);
        buffer[(x, bottom)].set_symbol("━").set_style(style);
    }
    buffer[(left, content)].set_symbol("┃").set_style(style);
    buffer[(right, content)].set_symbol("┃").set_style(style);
}

/// 绘制选中行/列的边框，覆盖在网格线之上、光标单元格边框之下。
fn draw_selection_border(
    frame: &mut Frame,
    area: Rect,
    selection: &Selection,
    scroll_row: usize,
    scroll_col: usize,
    visible_range: (usize, usize),
    style: Style,
) {
    let buffer = frame.buffer_mut();
    let (visible_rows, visible_cols) = visible_range;
    match *selection {
        Selection::Row(r) => {
            if r < scroll_row || r >= scroll_row + visible_rows {
                return;
            }
            let top = area.y + 1 + (r - scroll_row) as u16 * 2;
            let bottom = top + 2;
            if bottom >= area.bottom() {
                return;
            }
            for x in area.x..col_grid_x(area, visible_cols).min(area.right()) {
                buffer[(x, top)].set_symbol("━").set_style(style);
                buffer[(x, bottom)].set_symbol("━").set_style(style);
            }
        }
        Selection::Column(c) => {
            if c < scroll_col || c >= scroll_col + visible_cols {
                return;
            }
            let left = col_grid_x(area, c - scroll_col);
            let right = col_grid_x(area, c - scroll_col + 1);
            let top = area.y;
            let bottom = (area.y + 1 + visible_rows as u16 * 2).min(area.bottom());
            if left >= right || right > area.right() {
                return;
            }
            for y in top..bottom {
                buffer[(left, y)].set_symbol("┃").set_style(style);
                buffer[(right, y)].set_symbol("┃").set_style(style);
            }
        }
        Selection::Range { anchor, cursor } => {
            let c1 = anchor.col.min(cursor.col);
            let c2 = anchor.col.max(cursor.col);
            let r1 = anchor.row.min(cursor.row);
            let r2 = anchor.row.max(cursor.row);
            if r1 >= scroll_row + visible_rows || r2 < scroll_row {
                return;
            }
            if c1 >= scroll_col + visible_cols || c2 < scroll_col {
                return;
            }
            let vis_c1 = c1.max(scroll_col) - scroll_col;
            let vis_c2 = c2.min(scroll_col + visible_cols - 1) - scroll_col;
            let vis_r1 = r1.max(scroll_row) - scroll_row;
            let vis_r2 = r2.min(scroll_row + visible_rows - 1) - scroll_row;
            let top = area.y + 1 + vis_r1 as u16 * 2;
            let bottom = area.y + 1 + vis_r2 as u16 * 2 + 2;
            let left = col_grid_x(area, vis_c1);
            let right = col_grid_x(area, vis_c2 + 1);
            if left >= right || right > area.right() || bottom > area.bottom() {
                return;
            }
            buffer[(left, top)].set_symbol("┏").set_style(style);
            buffer[(right, top)].set_symbol("┓").set_style(style);
            buffer[(left, bottom)].set_symbol("┗").set_style(style);
            buffer[(right, bottom)].set_symbol("┛").set_style(style);
            for x in left + 1..right {
                buffer[(x, top)].set_symbol("━").set_style(style);
                buffer[(x, bottom)].set_symbol("━").set_style(style);
            }
            for y in top + 1..bottom {
                buffer[(left, y)].set_symbol("┃").set_style(style);
                buffer[(right, y)].set_symbol("┃").set_style(style);
            }
        }
    }
}

/// 从 HashMap 读取单元格并转成显示字符串
fn cell_text(wb: &Workbook, col: usize, row: usize) -> String {
    if let Some(cell) = wb.get_cell(CellAddress { row, col }) {
        match &cell.value {
            CellValue::Number(n) => n.to_string(),
            CellValue::Text(t) => t.clone(),
            CellValue::Empty => String::new(),
        }
    } else {
        String::new()
    }
}
