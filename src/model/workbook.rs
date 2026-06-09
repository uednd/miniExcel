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
    pub cells: HashMap<Coord, Cell>,
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
        self.cells.get(&coord)
    }

    pub fn set_cell(&mut self, coord: Coord, raw: String, value: CellValue) {
        self.cells.insert(coord, Cell { raw, value });
    }

    /// 删除指定行，该行上方的行不动，下方行上移，总行数减一。
    /// 至少保留一行，调用方负责裁剪光标位置。
    pub fn delete_row(&mut self, r: usize) {
        if self.rows <= 1 {
            return;
        }
        let mut new_cells: HashMap<Coord, Cell> = HashMap::new();
        for ((col, row), cell) in self.cells.drain() {
            if row < r {
                new_cells.insert((col, row), cell);
            } else if row > r {
                new_cells.insert((col, row - 1), cell);
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
        let mut new_cells: HashMap<Coord, Cell> = HashMap::new();
        for ((col, row), cell) in self.cells.drain() {
            if col < c {
                new_cells.insert((col, row), cell);
            } else if col > c {
                new_cells.insert((col - 1, row), cell);
            }
        }
        self.cells = new_cells;
        self.columns -= 1;
    }

    /// 清空指定行所有单元格内容。
    pub fn clear_row(&mut self, r: usize) {
        self.cells.retain(|&(_, row), _| row != r);
    }

    /// 清空指定列所有单元格内容。
    pub fn clear_column(&mut self, c: usize) {
        self.cells.retain(|&(col, _), _| col != c);
    }

    /// 清空矩形区域内所有单元格内容。
    pub fn clear_range(&mut self, c1: usize, r1: usize, c2: usize, r2: usize) {
        let (c_min, c_max) = (c1.min(c2), c1.max(c2));
        let (r_min, r_max) = (r1.min(r2), r1.max(r2));
        self.cells.retain(|&(col, row), _| {
            col < c_min || col > c_max || row < r_min || row > r_max
        });
    }
}
