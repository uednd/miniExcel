use crate::{model::workbook::Workbook, screen::ScreenCommand, theme::Theme};

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
}
