use crate::model::cell::CellAddress;

/// 表格视口状态：光标位置、滚动偏移、可见行列数。
///
/// 光标只能通过移动和裁剪方法修改，避免调用者绕过滚动同步。
/// 可见行列数由渲染阶段根据终端区域更新。
pub struct Viewport {
    cursor: CellAddress,
    scroll_row: usize,
    scroll_col: usize,
    visible_rows: usize,
    visible_cols: usize,
}

impl Viewport {
    /// 创建位于左上角、尚未测量可见范围的视口。
    pub fn new() -> Self {
        Self {
            cursor: CellAddress::new(0, 0),
            scroll_row: 0,
            scroll_col: 0,
            visible_rows: 0,
            visible_cols: 0,
        }
    }

    /// 当前光标地址。
    pub fn cursor(&self) -> CellAddress {
        self.cursor
    }

    /// 当前光标所在行。
    pub fn cursor_row(&self) -> usize {
        self.cursor.row
    }

    /// 当前光标所在列。
    pub fn cursor_col(&self) -> usize {
        self.cursor.col
    }

    /// 当前垂直滚动偏移。
    pub fn scroll_row(&self) -> usize {
        self.scroll_row
    }

    /// 当前水平滚动偏移。
    pub fn scroll_col(&self) -> usize {
        self.scroll_col
    }

    /// 当前可见行数。
    pub fn visible_rows(&self) -> usize {
        self.visible_rows
    }

    /// 当前可见列数。
    pub fn visible_cols(&self) -> usize {
        self.visible_cols
    }

    /// 更新渲染阶段测量出的可见行列数。
    /// 仅在终端尺寸变化时触发 scroll_into_view，避免每帧覆盖鼠标滚动。
    pub fn update_visible(&mut self, rows: usize, cols: usize) {
        let changed = rows != self.visible_rows || cols != self.visible_cols;
        self.visible_rows = rows;
        self.visible_cols = cols;
        if changed {
            self.scroll_into_view();
        }
    }

    /// 向上移动光标，并保持光标可见。
    pub fn move_up(&mut self) {
        if self.cursor.row > 0 {
            self.cursor.row -= 1;
            self.scroll_into_view();
        }
    }

    /// 向下移动光标，并保持光标可见。
    pub fn move_down(&mut self, row_count: usize) {
        if self.cursor.row + 1 < row_count {
            self.cursor.row += 1;
            self.scroll_into_view();
        }
    }

    /// 向左移动光标，并保持光标可见。
    pub fn move_left(&mut self) {
        if self.cursor.col > 0 {
            self.cursor.col -= 1;
            self.scroll_into_view();
        }
    }

    /// 向右移动光标，并保持光标可见。
    pub fn move_right(&mut self, col_count: usize) {
        if self.cursor.col + 1 < col_count {
            self.cursor.col += 1;
            self.scroll_into_view();
        }
    }

    /// 向上滚动指定行数。
    pub fn scroll_up(&mut self, amount: usize) {
        self.scroll_row = self.scroll_row.saturating_sub(amount);
    }

    /// 向下滚动指定行数，不超过工作簿底部。
    pub fn scroll_down(&mut self, amount: usize, row_count: usize) {
        let max = row_count.saturating_sub(self.visible_rows());
        self.scroll_row = (self.scroll_row + amount).min(max);
    }

    /// 向左滚动指定列数。
    pub fn scroll_left(&mut self, amount: usize) {
        self.scroll_col = self.scroll_col.saturating_sub(amount);
    }

    /// 向右滚动指定列数，不超过工作簿右边界。
    pub fn scroll_right(&mut self, amount: usize, col_count: usize) {
        let max = col_count.saturating_sub(self.visible_cols());
        self.scroll_col = (self.scroll_col + amount).min(max);
    }

    /// 确保光标在可见区域内。
    pub fn scroll_into_view(&mut self) {
        let rows = self.visible_rows;
        let cols = self.visible_cols;
        if rows == 0 || cols == 0 {
            return;
        }
        if self.cursor.row < self.scroll_row {
            self.scroll_row = self.cursor.row;
        }
        if self.cursor.row >= self.scroll_row + rows {
            self.scroll_row = self.cursor.row.saturating_sub(rows - 1);
        }
        if self.cursor.col < self.scroll_col {
            self.scroll_col = self.cursor.col;
        }
        if self.cursor.col >= self.scroll_col + cols {
            self.scroll_col = self.cursor.col.saturating_sub(cols - 1);
        }
    }

    /// 在删除行后裁剪光标行，并同步滚动位置。
    pub fn clamp_cursor_row(&mut self, row_count: usize) {
        if self.cursor.row >= row_count {
            self.cursor.row = row_count.saturating_sub(1);
            self.scroll_into_view();
        }
    }

    /// 在删除列后裁剪光标列，并同步滚动位置。
    pub fn clamp_cursor_col(&mut self, col_count: usize) {
        if self.cursor.col >= col_count {
            self.cursor.col = col_count.saturating_sub(1);
            self.scroll_into_view();
        }
    }
}
