use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

use super::cell::{Cell, CellValue, Coord};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workbook {
    pub version: u8,
    pub name: String,
    pub columns: usize,
    pub rows: usize,
    pub cells: HashMap<String, Cell>,
}

impl Workbook {
    pub fn new(name: String, columns: usize, rows: usize) -> Self {
        Self {
            version: 1,
            name,
            columns,
            rows,
            cells: HashMap::new(),
        }
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)
    }

    pub fn load<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let raw = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&raw)?)
    }

    pub fn get_cell(&self, coord: Coord) -> Option<&Cell> {
        let key = coord_key(coord);
        self.cells.get(&key)
    }

    pub fn set_cell(&mut self, coord: Coord, raw: String, value: CellValue) {
        let key = coord_key(coord);
        self.cells.insert(key, Cell { raw, value });
    }

    /// 删除指定行，该行上方的行不动，下方行上移，总行数减一。
    /// 至少保留一行，调用方负责裁剪光标位置。
    pub fn delete_row(&mut self, r: usize) {
        if self.rows <= 1 {
            return;
        }
        let mut new_cells: HashMap<String, Cell> = HashMap::new();
        for (key, cell) in self.cells.drain() {
            let (col, row) = parse_coord(&key);
            if row < r {
                new_cells.insert(key, cell);
            } else if row > r {
                new_cells.insert(coord_key((col, row - 1)), cell);
            }
        }
        self.cells = new_cells;
        self.rows -= 1;
    }

    /// 删除指定列，该列左侧的列不动，右侧列左移，总列数减一。
    /// 至少保留一列，调用方负责裁剪光标位置。
    pub fn delete_column(&mut self, c: usize) {
        if self.columns <= 1 {
            return;
        }
        let mut new_cells: HashMap<String, Cell> = HashMap::new();
        for (key, cell) in self.cells.drain() {
            let (col, row) = parse_coord(&key);
            if col < c {
                new_cells.insert(key, cell);
            } else if col > c {
                new_cells.insert(coord_key((col - 1, row)), cell);
            }
        }
        self.cells = new_cells;
        self.columns -= 1;
    }

}

/// Tuple 转 String 作为 HashMap 的 key，用于 JSON 序列化和反序列化
fn coord_key(coord: Coord) -> String {
    format!("{},{}", coord.0, coord.1)
}

/// 从 HashMap key ("col,row") 反向解析为 (col, row) 元组。
fn parse_coord(key: &str) -> Coord {
    let (col_str, row_str) = key.split_once(',').unwrap_or(("0", "0"));
    (
        col_str.parse().unwrap_or(0),
        row_str.parse().unwrap_or(0),
    )
}
