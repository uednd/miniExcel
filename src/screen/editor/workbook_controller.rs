use crate::model::{
    cell::{CellAddress, CellValue},
    document::{DocumentError, WorkbookDocument},
    workbook::{ClearSpec, Workbook},
};

use super::mode::Selection;

/// 当前选区的统计信息。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SelectionStats {
    pub average: f64,
    pub sum: f64,
    pub count: usize,
}

/// 工作簿业务操作入口。
///
/// 持有文档并负责单元格、区域、行列和保存操作；不处理按键和 UI 状态。
pub struct WorkbookController {
    document: WorkbookDocument,
}

impl WorkbookController {
    pub fn new(document: WorkbookDocument) -> Self {
        Self { document }
    }

    pub fn workbook(&self) -> &Workbook {
        self.document.workbook()
    }

    pub fn workbook_name(&self) -> &str {
        &self.document.workbook().name
    }

    pub fn row_count(&self) -> usize {
        self.document.workbook().rows
    }

    pub fn column_count(&self) -> usize {
        self.document.workbook().columns
    }

    pub fn current_cell_raw(&self, cursor: CellAddress) -> String {
        self.document
            .workbook()
            .get_cell(cursor)
            .map(|cell| cell.raw.clone())
            .unwrap_or_default()
    }

    pub fn commit_cell(&mut self, cursor: CellAddress, raw: String) {
        self.document.workbook_mut().set_text(cursor, raw);
    }

    pub fn clear_cell(&mut self, cursor: CellAddress) {
        self.document.workbook_mut().clear_cell(cursor);
    }

    pub fn save(&self) -> Result<(), DocumentError> {
        self.document.save()
    }

    pub fn clear_selection(&mut self, selection: Selection) {
        let spec = self.selection_clear_spec(selection);
        self.document.workbook_mut().clear_region(spec);
    }

    pub fn delete_row(&mut self, row: usize) {
        self.document.workbook_mut().delete_row(row);
    }

    pub fn delete_column(&mut self, col: usize) {
        self.document.workbook_mut().delete_column(col);
    }

    pub fn selection_stats(&self, selection: Selection) -> Option<SelectionStats> {
        let Selection::Range { .. } = selection else {
            return None;
        };

        let wb = self.document.workbook();
        let (r1, r2, c1, c2) = selection.normalized_bounds(wb.rows, wb.columns);
        if r1 == r2 && c1 == c2 {
            return None;
        }

        let mut sum = 0.0;
        let mut number_count = 0;
        let mut value_count = 0;

        for row in r1..=r2 {
            for col in c1..=c2 {
                let addr = CellAddress { row, col };
                let Some(cell) = wb.get_cell(addr) else {
                    continue;
                };

                value_count += 1;
                if let Some(number) = cell_value_as_number(&cell.value) {
                    sum += number;
                    number_count += 1;
                }
            }
        }

        let average = if number_count == 0 {
            0.0
        } else {
            sum / number_count as f64
        };

        Some(SelectionStats {
            average,
            sum,
            count: value_count,
        })
    }

    pub fn collect_selection(
        &self,
        selection: Option<Selection>,
        cursor: CellAddress,
    ) -> Vec<Vec<String>> {
        match selection {
            None => vec![vec![self.current_cell_raw(cursor)]],
            Some(selection) => {
                let wb = self.document.workbook();
                let (r1, r2, c1, c2) = selection.normalized_bounds(wb.rows, wb.columns);
                (r1..=r2)
                    .map(|r| (c1..=c2).map(|c| self.cell_raw_at(r, c)).collect())
                    .collect()
            }
        }
    }

    pub fn paste_range(&mut self, start: CellAddress, rows: &[Vec<String>]) -> Option<Selection> {
        if rows.is_empty() {
            return None;
        }

        let paste_cols = rows[0].len();
        let paste_rows = rows.len();

        self.document
            .workbook_mut()
            .ensure_size(start.row + paste_rows, start.col + paste_cols);

        for (r, row_cells) in rows.iter().enumerate() {
            let target_row = start.row + r;
            if target_row >= self.document.workbook().rows {
                break;
            }
            for (c, cell_text) in row_cells.iter().enumerate() {
                let target_col = start.col + c;
                if target_col >= self.document.workbook().columns {
                    break;
                }
                let addr = CellAddress {
                    row: target_row,
                    col: target_col,
                };
                if cell_text.is_empty() {
                    self.document.workbook_mut().clear_cell(addr);
                } else {
                    self.document
                        .workbook_mut()
                        .set_text(addr, cell_text.clone());
                }
            }
        }

        let end_row = (start.row + paste_rows)
            .min(self.document.workbook().rows)
            .saturating_sub(1);
        let end_col = (start.col + paste_cols)
            .min(self.document.workbook().columns)
            .saturating_sub(1);

        Some(Selection::Range {
            anchor: start,
            cursor: CellAddress {
                row: end_row,
                col: end_col,
            },
        })
    }

    fn selection_clear_spec(&self, selection: Selection) -> ClearSpec {
        let wb = self.document.workbook();
        let (r1, r2, c1, c2) = selection.normalized_bounds(wb.rows, wb.columns);
        match selection {
            Selection::Row(r) => ClearSpec::Row(r),
            Selection::Column(c) => ClearSpec::Column(c),
            Selection::Range { .. } => ClearSpec::Rect { c1, r1, c2, r2 },
        }
    }

    fn cell_raw_at(&self, row: usize, col: usize) -> String {
        self.document
            .workbook()
            .get_cell(CellAddress { row, col })
            .map(|cell| cell.raw.clone())
            .unwrap_or_default()
    }
}

fn cell_value_as_number(value: &CellValue) -> Option<f64> {
    match value {
        CellValue::Number(n) => Some(*n),
        CellValue::Text(text) => text.parse::<f64>().ok(),
        CellValue::Empty | CellValue::Error(_) => None,
    }
}
