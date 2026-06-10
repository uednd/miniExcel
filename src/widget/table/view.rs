use crate::model::cell::CellAddress;

/// 渲染所需的滚动与视口状态，不依赖 `screen::editor::Viewport`。
#[derive(Debug, Clone, Copy)]
pub struct GridScroll {
    pub scroll_row: usize,
    pub scroll_col: usize,
    pub visible_rows: usize,
    pub visible_cols: usize,
    pub cursor: CellAddress,
    pub total_rows: usize,
    pub total_cols: usize,
}

/// 预归一化的选区，供绘制区域边框使用。
///
/// `Range` 的边界已排序（min ≤ max），渲染层无需再调用 `Selection::normalized()`。
#[derive(Debug, Clone, Copy)]
pub enum GridSelection {
    Row(usize),
    Column(usize),
    Range {
        min_row: usize,
        max_row: usize,
        min_col: usize,
        max_col: usize,
    },
}

impl GridSelection {
    pub fn contains(&self, addr: CellAddress) -> bool {
        match *self {
            GridSelection::Row(r) => r == addr.row,
            GridSelection::Column(c) => c == addr.col,
            GridSelection::Range {
                min_row,
                max_row,
                min_col,
                max_col,
            } => {
                addr.row >= min_row
                    && addr.row <= max_row
                    && addr.col >= min_col
                    && addr.col <= max_col
            }
        }
    }
}
