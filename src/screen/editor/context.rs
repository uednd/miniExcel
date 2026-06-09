use std::cell::Cell;

use crate::{model::workbook::Workbook, theme::Theme};

use super::mode::Selection;

/// 所有编辑模式共享的上下文
pub struct TableContext {
    pub theme: Theme,
    pub path: String,
    pub wb: Workbook,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub scroll_row: usize,
    pub scroll_col: usize,
    pub visible_rows: Cell<usize>,
    pub visible_cols: Cell<usize>,
    pub selection: Option<Selection>,
}

impl TableContext {
    pub fn save(&self) {
        let _ = self.wb.save(&self.path);
    }

    /// 滚动视口以确保光标可见
    pub fn scroll_into_view(&mut self) {
        let visible_rows = self.visible_rows.get();
        let visible_cols = self.visible_cols.get();
        if self.cursor_row < self.scroll_row {
            self.scroll_row = self.cursor_row;
        }
        if self.cursor_row >= self.scroll_row + visible_rows {
            self.scroll_row = self.cursor_row.saturating_sub(visible_rows - 1);
        }
        if self.cursor_col < self.scroll_col {
            self.scroll_col = self.cursor_col;
        }
        if self.cursor_col >= self.scroll_col + visible_cols {
            self.scroll_col = self.cursor_col.saturating_sub(visible_cols - 1);
        }
    }
}
