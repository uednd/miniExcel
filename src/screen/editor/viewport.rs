use crate::model::cell::CellAddress;

/// 表格视口状态：光标位置、滚动偏移、可见行/列数。
pub struct Viewport {
    cursor: CellAddress,
    scroll_row: usize,
    scroll_col: usize,
    visible_rows: usize,
    visible_cols: usize,
}

impl Viewport {
    pub fn new() -> Self {
        Self {
            cursor: CellAddress::new(0, 0),
            scroll_row: 0,
            scroll_col: 0,
            visible_rows: 0,
            visible_cols: 0,
        }
    }

    // ── visible 读写 ──

    pub fn cursor(&self) -> CellAddress {
        self.cursor
    }

    pub fn cursor_row(&self) -> usize {
        self.cursor.row
    }

    pub fn cursor_col(&self) -> usize {
        self.cursor.col
    }

    pub fn scroll_row(&self) -> usize {
        self.scroll_row
    }

    pub fn scroll_col(&self) -> usize {
        self.scroll_col
    }

    pub fn visible_rows(&self) -> usize {
        self.visible_rows
    }

    pub fn visible_cols(&self) -> usize {
        self.visible_cols
    }

    /// 渲染阶段调用：由外部根据终端区域计算后写入。
    pub fn update_visible(&mut self, rows: usize, cols: usize) {
        self.visible_rows = rows;
        self.visible_cols = cols;
    }

    // ── 光标移动（自动聚焦） ──

    pub fn move_up(&mut self) {
        if self.cursor.row > 0 {
            self.cursor.row -= 1;
            self.scroll_into_view();
        }
    }

    pub fn move_down(&mut self, row_count: usize) {
        if self.cursor.row + 1 < row_count {
            self.cursor.row += 1;
            self.scroll_into_view();
        }
    }

    pub fn move_left(&mut self) {
        if self.cursor.col > 0 {
            self.cursor.col -= 1;
            self.scroll_into_view();
        }
    }

    pub fn move_right(&mut self, col_count: usize) {
        if self.cursor.col + 1 < col_count {
            self.cursor.col += 1;
            self.scroll_into_view();
        }
    }

    // ── 滚动 ──

    pub fn scroll_up(&mut self, amount: usize) {
        self.scroll_row = self.scroll_row.saturating_sub(amount);
    }

    pub fn scroll_down(&mut self, amount: usize, row_count: usize) {
        let max = row_count.saturating_sub(self.visible_rows());
        self.scroll_row = (self.scroll_row + amount).min(max);
    }

    pub fn scroll_left(&mut self, amount: usize) {
        self.scroll_col = self.scroll_col.saturating_sub(amount);
    }

    pub fn scroll_right(&mut self, amount: usize, col_count: usize) {
        let max = col_count.saturating_sub(self.visible_cols());
        self.scroll_col = (self.scroll_col + amount).min(max);
    }

    /// 确保光标在可见区域内。
    pub fn scroll_into_view(&mut self) {
        let rows = self.visible_rows;
        let cols = self.visible_cols;
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

    // ── 光标裁剪 ──

    pub fn clamp_cursor_row(&mut self, row_count: usize) {
        if self.cursor.row >= row_count {
            self.cursor.row = row_count.saturating_sub(1);
        }
    }

    pub fn clamp_cursor_col(&mut self, col_count: usize) {
        if self.cursor.col >= col_count {
            self.cursor.col = col_count.saturating_sub(1);
        }
    }
}
