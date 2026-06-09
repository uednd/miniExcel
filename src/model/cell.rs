use std::sync::LazyLock;

use serde::{Deserialize, Serialize};

use super::limits::MAX_COLUMNS;

/// 单元格坐标（行，列）
pub type Coord = (usize, usize);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cell {
    pub raw: String,
    pub value: CellValue, // TODO：公式解析
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CellValue {
    Number(f64),
    Text(String),
    Empty,
}

static COL_NAMES: LazyLock<Vec<String>> = LazyLock::new(|| {
    let mut v = Vec::with_capacity(MAX_COLUMNS);
    for i in 0..MAX_COLUMNS {
        if i < 26 {
            v.push(char::from_u32(b'A' as u32 + i as u32).unwrap().to_string());
        } else {
            let first = char::from_u32(b'A' as u32 + (i / 26 - 1) as u32).unwrap();
            let second = char::from_u32(b'A' as u32 + (i % 26) as u32).unwrap();
            v.push(format!("{}{}", first, second));
        }
    }
    v
});

/// 列索引转字母
pub fn col_name(index: usize) -> &'static str {
    COL_NAMES.get(index).map(|s| s.as_str()).unwrap_or("?")
}

/// 行列索引转坐标(A1, B2)，用于 status_hint
pub fn display_coord(row: usize, col: usize) -> String {
    format!("{}{}", col_name(col), row + 1)
}
