use std::sync::LazyLock;

use serde::{Deserialize, Serialize};

use super::formula::error::CellError;
use super::limits::MAX_COLUMNS;

/// 单元格地址，使用从 0 开始的行列索引。
///
/// 字段顺序固定为 `row, col`，显示时再转换为 A1/B2 形式。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CellAddress {
    pub row: usize,
    pub col: usize,
}

impl CellAddress {
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }

    /// 转换为用户可见的坐标文本，如 A1、B2。
    pub fn display(&self) -> String {
        format!("{}{}", col_name(self.col), self.row + 1)
    }
}

/// 单元格保存用户输入的原始文本和解析后的值。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cell {
    pub raw: String,
    pub value: CellValue,
}

/// 单元格当前可渲染的值。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CellValue {
    Number(f64),
    Text(String),
    Empty,
    Error(CellError),
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

/// 将从 0 开始的列索引转换为 Excel 风格列名。
///
/// 超出 `MAX_COLUMNS` 时返回 `"?"`。
pub fn col_name(index: usize) -> &'static str {
    COL_NAMES.get(index).map(|s| s.as_str()).unwrap_or("?")
}
