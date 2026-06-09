use crate::{model::workbook::Workbook, screen::ScreenCommand, theme::Theme};

use super::{mode::Selection, viewport::Viewport};

/// 所有编辑模式共享的上下文
pub struct TableContext {
    pub theme: Theme,
    pub path: String,
    pub wb: Workbook,
    pub viewport: Viewport,
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

    pub fn delete_current_row(&mut self) {
        self.wb.delete_row(self.viewport.cursor_row());
        self.viewport.clamp_cursor_row(self.wb.rows);
        self.viewport.scroll_into_view();
    }

    pub fn delete_current_column(&mut self) {
        self.wb.delete_column(self.viewport.cursor_col());
        self.viewport.clamp_cursor_col(self.wb.columns);
        self.viewport.scroll_into_view();
    }
}
