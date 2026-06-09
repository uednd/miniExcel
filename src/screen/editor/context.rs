use crate::{model::workbook::Workbook, screen::ScreenCommand, theme::Theme};

use super::{mode::Selection, viewport::Viewport};

/// 编辑器屏幕共享状态。
///
/// `TableContext` 持有工作簿、视口和选区。模式可以读取这些状态，
/// 但涉及多步不变量的操作应优先通过方法完成，例如删除当前行列。
pub struct TableContext {
    pub theme: Theme,
    pub path: String,
    pub wb: Workbook,
    pub viewport: Viewport,
    pub selection: Option<Selection>,
    /// 模式需要切换屏幕时写入，随后由 `ModeHost` 取走。
    pub pending_command: Option<ScreenCommand>,
}

impl TableContext {
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
}
