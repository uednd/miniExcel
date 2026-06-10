use crate::{
    model::{
        cell::{CellAddress, CellValue},
        document::WorkbookDocument,
        workbook::{ClearSpec, Workbook},
    },
    theme::Theme,
};

use super::{mode::Selection, viewport::Viewport};

/// 编辑器屏幕共享状态。
///
/// `TableContext` 持有工作簿、视口和选区。模式可以读取这些状态，
/// 但涉及多步不变量的操作应优先通过方法完成，例如删除当前行列。
pub struct TableContext {
    pub theme: Theme,
    pub viewport: Viewport,
    document: WorkbookDocument,
    selection: Option<Selection>,
    copied_region: Option<Selection>,
    blink_visible: bool,
    status_message: Option<String>,
}

/// 当前选区的统计信息。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SelectionStats {
    pub average: f64,
    pub sum: f64,
    pub count: usize,
}

impl TableContext {
    /// 创建编辑器共享状态。
    pub fn new(theme: Theme, document: WorkbookDocument) -> Self {
        Self {
            theme,
            document,
            viewport: Viewport::new(),
            selection: None,
            copied_region: None,
            blink_visible: true,
            status_message: None,
        }
    }

    /// 当前工作簿。
    pub fn workbook(&self) -> &Workbook {
        self.document.workbook()
    }

    /// 当前工作簿名。
    pub fn workbook_name(&self) -> &str {
        &self.document.workbook().name
    }

    /// 当前工作簿行数。
    pub fn row_count(&self) -> usize {
        self.document.workbook().rows
    }

    /// 当前工作簿列数。
    pub fn column_count(&self) -> usize {
        self.document.workbook().columns
    }

    /// 当前选区。
    pub fn selection(&self) -> Option<&Selection> {
        self.selection.as_ref()
    }

    /// 设置当前选区。
    pub fn set_selection(&mut self, selection: Selection) {
        self.selection = Some(selection);
    }

    /// 清除当前选区。
    pub fn clear_selection(&mut self) {
        self.selection = None;
    }

    /// 统计当前多单元格矩形选区。
    ///
    /// `count` 统计有内容的单元格；`sum` 和 `average` 只统计数字。
    pub fn selection_stats(&self) -> Option<SelectionStats> {
        let selection = self.selection?;
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

    /// 已复制的区域（用于渲染虚线边框）。
    ///
    /// `None` 表示没有待粘贴的复制内容。
    pub fn copied_region(&self) -> Option<&Selection> {
        self.copied_region.as_ref()
    }

    /// 设置当前帧的闪烁可见性。
    pub fn set_blink_visible(&mut self, visible: bool) {
        self.blink_visible = visible;
    }

    /// 当前帧的闪烁可见性。
    pub fn blink_visible(&self) -> bool {
        self.blink_visible
    }

    /// 光标所在单元格的原始文本。
    pub fn current_cell_raw(&self) -> String {
        self.document
            .workbook()
            .get_cell(self.viewport.cursor())
            .map(|cell| cell.raw.clone())
            .unwrap_or_default()
    }

    /// 保存当前工作簿。
    pub fn save(&mut self) -> bool {
        match self.document.save() {
            Ok(()) => {
                self.status_message = Some(String::from("已保存"));
                true
            }
            Err(err) => {
                self.status_message = Some(err.message());
                false
            }
        }
    }

    pub fn status_message(&self) -> Option<&str> {
        self.status_message.as_deref()
    }

    /// 设置底部状态栏消息。
    pub fn set_status_message(&mut self, message: impl Into<String>) {
        self.status_message = Some(message.into());
    }

    /// 写入光标所在单元格的文本。
    pub fn commit_current_cell(&mut self, raw: String) {
        self.document
            .workbook_mut()
            .set_text(self.viewport.cursor(), raw);
    }

    /// 清除光标所在单元格。
    pub fn clear_current_cell(&mut self) {
        self.document
            .workbook_mut()
            .clear_cell(self.viewport.cursor());
    }

    /// 复制当前选区的单元格内容到系统剪贴板。
    ///
    /// - 无选区 → 复制光标所在单元格
    /// - Range   → 复制矩形区域内所有单元格
    /// - Row     → 复制整行（所有列）
    /// - Column  → 复制整列（所有行）
    ///
    /// 不改变选区或光标状态。
    pub fn copy_selection(&mut self) -> Result<(), String> {
        let cells = self.collect_selection();
        let tsv = crate::clipboard::to_tsv(&cells);
        crate::clipboard::copy_to_clipboard(&tsv)?;
        self.copied_region = match &self.selection {
            Some(sel) => Some(*sel),
            None => {
                let cur = self.viewport.cursor();
                Some(Selection::Range {
                    anchor: cur,
                    cursor: cur,
                })
            }
        };
        Ok(())
    }

    /// 从系统剪贴板粘贴到光标位置。
    ///
    /// 自动扩展表格行列（不超过最大限制）。
    /// 粘贴后创建 Range 选区覆盖整个粘贴区域。
    /// 光标位置不变。
    pub fn paste_from_clipboard(&mut self) -> Result<(), String> {
        let text = crate::clipboard::read_from_clipboard()?;
        let rows = crate::clipboard::from_tsv(&text);
        if rows.is_empty() {
            return Ok(());
        }

        let paste_cols = rows[0].len();
        let paste_rows = rows.len();
        let start = self.viewport.cursor();

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

        self.copied_region = None;
        self.set_selection(Selection::Range {
            anchor: start,
            cursor: CellAddress {
                row: end_row,
                col: end_col,
            },
        });

        Ok(())
    }

    /// 清空当前选区，并退出选区状态。
    pub fn clear_selection_cells(&mut self) {
        if let Some(spec) = self.selection_clear_spec() {
            self.document.workbook_mut().clear_region(spec);
            self.selection = None;
        }
    }

    /// 删除光标所在行，并同步裁剪光标和滚动位置。
    pub fn delete_current_row(&mut self) {
        self.document
            .workbook_mut()
            .delete_row(self.viewport.cursor_row());
        self.viewport
            .clamp_cursor_row(self.document.workbook().rows);
    }

    /// 删除光标所在列，并同步裁剪光标和滚动位置。
    pub fn delete_current_column(&mut self) {
        self.document
            .workbook_mut()
            .delete_column(self.viewport.cursor_col());
        self.viewport
            .clamp_cursor_col(self.document.workbook().columns);
    }

    fn selection_clear_spec(&self) -> Option<ClearSpec> {
        self.selection.as_ref().map(|selection| {
            let wb = self.document.workbook();
            let (r1, r2, c1, c2) = selection.normalized_bounds(wb.rows, wb.columns);
            match *selection {
                Selection::Row(r) => ClearSpec::Row(r),
                Selection::Column(c) => ClearSpec::Column(c),
                Selection::Range { .. } => ClearSpec::Rect { c1, r1, c2, r2 },
            }
        })
    }

    fn collect_selection(&self) -> Vec<Vec<String>> {
        match &self.selection {
            None => {
                vec![vec![self.current_cell_raw()]]
            }
            Some(selection) => {
                let wb = self.document.workbook();
                let (r1, r2, c1, c2) = selection.normalized_bounds(wb.rows, wb.columns);
                (r1..=r2)
                    .map(|r| (c1..=c2).map(|c| self.cell_raw_at(r, c)).collect())
                    .collect()
            }
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
