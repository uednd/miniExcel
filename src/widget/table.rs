use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::{Cell, Row, Table},
};

use crate::{
    model::{
        cell::{CellValue, col_name},
        workbook::Workbook,
    },
    theme::Theme,
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
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub theme: Theme,
    pub edit_buffer: Option<&'a str>,
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

        // 表头行
        let mut header_cells = vec![Cell::from("").style(header_style)];
        for ci in cfg.scroll_col..cfg.scroll_col + visible_cols {
            // 防止列数超过工作表实际列数导致的越界
            if ci >= cfg.wb.columns {
                break;
            }
            // 光标列高亮
            let style = if ci == cfg.cursor_col {
                selected_header_style
            } else {
                header_style
            };
            header_cells.push(
                Cell::from(Line::from(col_name(ci)).alignment(Alignment::Center)).style(style),
            );
        }

        // 数据行
        let mut rows: Vec<Row> = Vec::new();
        for ri in cfg.scroll_row..cfg.scroll_row + visible_rows {
            if ri >= cfg.wb.rows {
                break;
            }

            let label_style = if ri == cfg.cursor_row {
                selected_header_style
            } else {
                header_style
            };
            // 行号
            let mut content_cells = vec![
                Cell::from(Line::from((ri + 1).to_string()).alignment(Alignment::Right))
                    .style(label_style),
            ];
            for ci in cfg.scroll_col..cfg.scroll_col + visible_cols {
                if ci >= cfg.wb.columns {
                    break;
                }
                let text = if ci == cfg.cursor_col && ri == cfg.cursor_row {
                    // 编辑模式
                    if let Some(buffer) = cfg.edit_buffer {
                        if buffer.is_empty() {
                            "█".to_string()
                        } else {
                            format!("{}█", buffer)
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

        // 绘制选中单元格高亮边框
        if cfg.cursor_row >= cfg.scroll_row
            && cfg.cursor_row < cfg.scroll_row + visible_rows
            && cfg.cursor_col >= cfg.scroll_col
            && cfg.cursor_col < cfg.scroll_col + visible_cols
        {
            draw_cell_border(
                frame,
                area,
                cfg.cursor_row - cfg.scroll_row,
                cfg.cursor_col - cfg.scroll_col,
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

/// 从 HashMap 读取单元格并转成显示字符串
fn cell_text(wb: &Workbook, col: usize, row: usize) -> String {
    if let Some(cell) = wb.get_cell((col, row)) {
        match &cell.value {
            CellValue::Number(n) => n.to_string(),
            CellValue::Text(t) => t.clone(),
            CellValue::Empty => String::new(),
        }
    } else {
        String::new()
    }
}
