use std::sync::LazyLock;

use serde::{Deserialize, Serialize};

use super::limits::MAX_COLUMNS;

/// 单元格地址（行，列）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CellAddress {
    pub row: usize,
    pub col: usize,
}

impl CellAddress {
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }

    /// 转显示格式 (A1, B2...)
    pub fn display(&self) -> String {
        format!("{}{}", col_name(self.col), self.row + 1)
    }
}

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


