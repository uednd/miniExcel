use serde::{Deserialize, Serialize};

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

const COL_NAMES: [&str; 26] = [
    "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S",
    "T", "U", "V", "W", "X", "Y", "Z",
];

/// 列索引转字母
pub fn col_name(index: usize) -> &'static str {
    COL_NAMES.get(index).copied().unwrap_or("?")
}

/// 行列索引转坐标(A1, B2)，用于 status_hint
pub fn display_coord(row: usize, col: usize) -> String {
    format!("{}{}", col_name(col), row + 1)
}
