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

/// 列索引转字母
pub fn col_name(index: usize) -> String {
    char::from_u32(b'A' as u32 + index as u32)
        .unwrap()
        .to_string()
}

/// 行列索引转坐标(A1, B2)，用于 status_hint
pub fn display_coord(row: usize, col: usize) -> String {
    format!("{}{}", col_name(col), row + 1)
}
