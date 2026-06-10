use crate::{
    model::{cell::CellAddress, workbook::Workbook},
    screen::ScreenCommand,
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
    path: String,
    wb: Workbook,
    selection: Option<Selection>,
    copied_region: Option<Selection>,
    /// 模式需要切换屏幕时写入，随后由 `ModeHost` 取走。
    pending_command: Option<ScreenCommand>,
}

impl TableContext {
    /// 创建编辑器共享状态。
    pub fn new(theme: Theme, path: String, wb: Workbook) -> Self {
        Self {
            theme,
            path,
            wb,
            viewport: Viewport::new(),
            selection: None,
            copied_region: None,
            pending_command: None,
        }
    }

    /// 当前工作簿。
    pub fn workbook(&self) -> &Workbook {
        &self.wb
    }

    /// 当前工作簿名。
    pub fn workbook_name(&self) -> &str {
        &self.wb.name
    }

    /// 当前工作簿行数。
    pub fn row_count(&self) -> usize {
        self.wb.rows
    }

    /// 当前工作簿列数。
    pub fn column_count(&self) -> usize {
        self.wb.columns
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

    /// 已复制的区域（用于渲染虚线边框）。
    ///
    /// `None` 表示没有待粘贴的复制内容。
    pub fn copied_region(&self) -> Option<&Selection> {
        self.copied_region.as_ref()
    }

    /// 光标所在单元格的原始文本。
    pub fn current_cell_raw(&self) -> String {
        self.wb
            .get_cell(self.viewport.cursor())
            .map(|cell| cell.raw.clone())
            .unwrap_or_default()
    }

    /// 保存当前工作簿。
    ///
    /// 保存失败会被忽略；当前界面还没有错误提示位置。
    pub fn save(&self) {
        let _ = self.wb.save(&self.path);
    }

    /// 请求编辑器返回首页。
    pub fn go_home(&mut self) {
        self.pending_command = Some(ScreenCommand::GoHome);
    }

    /// 取走待处理的屏幕命令。
    pub fn take_pending_command(&mut self) -> Option<ScreenCommand> {
        self.pending_command.take()
    }

    /// 写入光标所在单元格的文本。
    pub fn set_current_cell_text(&mut self, raw: String) {
        self.wb.set_text(self.viewport.cursor(), raw);
    }

    /// 清除光标所在单元格。
    pub fn clear_current_cell(&mut self) {
        self.wb.clear_cell(self.viewport.cursor());
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

        self.wb
            .ensure_size(start.row + paste_rows, start.col + paste_cols);

        for (r, row_cells) in rows.iter().enumerate() {
            let target_row = start.row + r;
            if target_row >= self.wb.rows {
                break;
            }
            for (c, cell_text) in row_cells.iter().enumerate() {
                let target_col = start.col + c;
                if target_col >= self.wb.columns {
                    break;
                }
                let addr = CellAddress {
                    row: target_row,
                    col: target_col,
                };
                if cell_text.is_empty() {
                    self.wb.clear_cell(addr);
                } else {
                    self.wb.set_text(addr, cell_text.clone());
                }
            }
        }

        let end_row = (start.row + paste_rows).min(self.wb.rows).saturating_sub(1);
        let end_col = (start.col + paste_cols)
            .min(self.wb.columns)
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
            self.wb.clear_region(spec);
            self.selection = None;
        }
    }

    /// 删除光标所在行，并同步裁剪光标和滚动位置。
    pub fn delete_current_row(&mut self) {
        self.wb.delete_row(self.viewport.cursor_row());
        self.viewport.clamp_cursor_row(self.wb.rows);
        self.viewport.scroll_into_view();
    }

    /// 删除光标所在列，并同步裁剪光标和滚动位置。
    pub fn delete_current_column(&mut self) {
        self.wb.delete_column(self.viewport.cursor_col());
        self.viewport.clamp_cursor_col(self.wb.columns);
        self.viewport.scroll_into_view();
    }

    fn selection_clear_spec(&self) -> Option<crate::model::workbook::ClearSpec> {
        use crate::model::workbook::ClearSpec;

        self.selection.as_ref().map(|selection| match *selection {
            Selection::Row(r) => ClearSpec::Row(r),
            Selection::Column(c) => ClearSpec::Column(c),
            Selection::Range { anchor, cursor } => ClearSpec::Rect {
                c1: anchor.col.min(cursor.col),
                r1: anchor.row.min(cursor.row),
                c2: anchor.col.max(cursor.col),
                r2: anchor.row.max(cursor.row),
            },
        })
    }

    fn collect_selection(&self) -> Vec<Vec<String>> {
        match &self.selection {
            None => {
                vec![vec![self.current_cell_raw()]]
            }
            Some(Selection::Range { anchor, cursor }) => {
                let r1 = anchor.row.min(cursor.row);
                let r2 = anchor.row.max(cursor.row);
                let c1 = anchor.col.min(cursor.col);
                let c2 = anchor.col.max(cursor.col);
                (r1..=r2)
                    .map(|r| (c1..=c2).map(|c| self.cell_raw_at(r, c)).collect())
                    .collect()
            }
            Some(Selection::Row(r)) => {
                let r = *r;
                vec![
                    (0..self.wb.columns)
                        .map(|c| self.cell_raw_at(r, c))
                        .collect(),
                ]
            }
            Some(Selection::Column(c)) => {
                let c = *c;
                (0..self.wb.rows)
                    .map(|r| vec![self.cell_raw_at(r, c)])
                    .collect()
            }
        }
    }

    fn cell_raw_at(&self, row: usize, col: usize) -> String {
        self.wb
            .get_cell(CellAddress { row, col })
            .map(|cell| cell.raw.clone())
            .unwrap_or_default()
    }
}
