use crate::model::cell::CellAddress;

use super::{
    mode::{Direction, Selection},
    viewport::Viewport,
};

/// 编辑器 UI 状态。
///
/// 只保存视口、选区、复制区域、闪烁和状态栏消息，不知道工作簿内容。
pub struct EditorState {
    viewport: Viewport,
    selection: Option<Selection>,
    copied_region: Option<Selection>,
    blink_visible: bool,
    status_message: Option<String>,
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            viewport: Viewport::new(),
            selection: None,
            copied_region: None,
            blink_visible: true,
            status_message: None,
        }
    }

    pub fn viewport(&self) -> &Viewport {
        &self.viewport
    }

    pub fn cursor(&self) -> CellAddress {
        self.viewport.cursor()
    }

    pub fn selection(&self) -> Option<&Selection> {
        self.selection.as_ref()
    }

    pub fn copied_region(&self) -> Option<&Selection> {
        self.copied_region.as_ref()
    }

    pub fn set_copied_region(&mut self, selection: Selection) {
        self.copied_region = Some(selection);
    }

    pub fn clear_copied_region(&mut self) {
        self.copied_region = None;
    }

    pub fn set_selection(&mut self, selection: Selection) {
        self.selection = Some(selection);
    }

    pub fn clear_selection(&mut self) {
        self.selection = None;
    }

    pub fn blink_visible(&self) -> bool {
        self.blink_visible
    }

    pub fn set_blink_visible(&mut self, visible: bool) {
        self.blink_visible = visible;
    }

    pub fn status_message(&self) -> Option<&str> {
        self.status_message.as_deref()
    }

    pub fn set_status_message(&mut self, message: impl Into<String>) {
        self.status_message = Some(message.into());
    }

    pub fn update_visible_capacity(&mut self, rows: usize, cols: usize) {
        self.viewport.update_visible(rows, cols);
    }

    pub fn scroll_up(&mut self, rows: usize) {
        self.viewport.scroll_up(rows);
    }

    pub fn scroll_down(&mut self, rows: usize, row_count: usize) {
        self.viewport.scroll_down(rows, row_count);
    }

    pub fn scroll_left(&mut self, cols: usize) {
        self.viewport.scroll_left(cols);
    }

    pub fn scroll_right(&mut self, cols: usize, column_count: usize) {
        self.viewport.scroll_right(cols, column_count);
    }

    pub fn move_cursor(&mut self, direction: Direction, row_count: usize, column_count: usize) {
        match direction {
            Direction::Up => self.viewport.move_up(),
            Direction::Down => self.viewport.move_down(row_count),
            Direction::Left => self.viewport.move_left(),
            Direction::Right => self.viewport.move_right(column_count),
        }
    }

    pub fn move_cursor_and_clear_selection(
        &mut self,
        direction: Direction,
        row_count: usize,
        column_count: usize,
    ) {
        self.clear_selection();
        self.move_cursor(direction, row_count, column_count);
    }

    pub fn start_range_selection(
        &mut self,
        direction: Direction,
        row_count: usize,
        column_count: usize,
    ) {
        let anchor = self.cursor();
        self.move_cursor(direction, row_count, column_count);
        self.selection = Some(Selection::Range {
            anchor,
            cursor: self.cursor(),
        });
    }

    pub fn extend_range_selection(
        &mut self,
        direction: Direction,
        row_count: usize,
        column_count: usize,
    ) {
        let Some(Selection::Range { anchor, .. }) = self.selection else {
            return;
        };
        self.move_cursor(direction, row_count, column_count);
        self.selection = Some(Selection::Range {
            anchor,
            cursor: self.cursor(),
        });
    }

    pub fn select_current_row(&mut self) {
        self.selection = Some(Selection::Row(self.cursor().row));
    }

    pub fn select_current_column(&mut self) {
        self.selection = Some(Selection::Column(self.cursor().col));
    }

    pub fn clamp_cursor_row(&mut self, row_count: usize) {
        self.viewport.clamp_cursor_row(row_count);
    }

    pub fn clamp_cursor_col(&mut self, column_count: usize) {
        self.viewport.clamp_cursor_col(column_count);
    }
}
