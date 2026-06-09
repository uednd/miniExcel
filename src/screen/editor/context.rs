use std::cell::Cell;

use crate::{model::{cell::CellAddress, workbook::Workbook}, screen::ScreenCommand, theme::Theme};

use super::mode::Selection;

/// 所有编辑模式共享的上下文
pub struct TableContext {
    pub theme: Theme,
    pub path: String,
    pub wb: Workbook,
    pub cursor: CellAddress,
    pub scroll_row: usize,
    pub scroll_col: usize,
    pub visible_rows: Cell<usize>,
    pub visible_cols: Cell<usize>,
    pub selection: Option<Selection>,
    pub pending_command: Option<ScreenCommand>,
}

impl TableContext {
    pub fn save(&self) {
        let _ = self.wb.save(&self.path);
    }

    pub fn go_home(&mut self) {
        self.pending_command = Some(ScreenCommand::GoHome);
    }

    pub fn take_pending_command(&mut self) -> Option<ScreenCommand> {
        self.pending_command.take()
    }

    /// 滚动视口以确保光标可见
    pub fn scroll_into_view(&mut self) {
        let visible_rows = self.visible_rows.get();
        let visible_cols = self.visible_cols.get();
        if self.cursor.row < self.scroll_row {
            self.scroll_row = self.cursor.row;
        }
        if self.cursor.row >= self.scroll_row + visible_rows {
            self.scroll_row = self.cursor.row.saturating_sub(visible_rows - 1);
        }
        if self.cursor.col < self.scroll_col {
            self.scroll_col = self.cursor.col;
        }
        if self.cursor.col >= self.scroll_col + visible_cols {
            self.scroll_col = self.cursor.col.saturating_sub(visible_cols - 1);
        }
    }
}
