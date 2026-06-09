use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

use super::cell::{Cell, CellAddress, CellValue};

/// 清空区域的规格。
#[derive(Debug, Clone)]
pub enum ClearSpec {
    Row(usize),
    Column(usize),
    Rect {
        c1: usize,
        r1: usize,
        c2: usize,
        r2: usize,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workbook {
    pub version: u8,
    pub name: String,
    pub columns: usize,
    pub rows: usize,
    pub cells: HashMap<CellAddress, Cell>,
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

    pub fn get_cell(&self, addr: CellAddress) -> Option<&Cell> {
        self.cells.get(&addr)
    }

    pub fn set_cell(&mut self, addr: CellAddress, raw: String, value: CellValue) {
        self.cells.insert(addr, Cell { raw, value });
    }

    /// 删除指定行，该行上方的行不动，下方行上移，总行数减一。
    /// 至少保留一行，调用方负责裁剪光标位置。
    pub fn delete_row(&mut self, r: usize) {
        if self.rows <= 1 {
            return;
        }
        let mut new_cells: HashMap<CellAddress, Cell> = HashMap::new();
        for (addr, cell) in self.cells.drain() {
            if addr.row < r {
                new_cells.insert(addr, cell);
            } else if addr.row > r {
                new_cells.insert(
                    CellAddress {
                        row: addr.row - 1,
                        col: addr.col,
                    },
                    cell,
                );
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
        let mut new_cells: HashMap<CellAddress, Cell> = HashMap::new();
        for (addr, cell) in self.cells.drain() {
            if addr.col < c {
                new_cells.insert(addr, cell);
            } else if addr.col > c {
                new_cells.insert(
                    CellAddress {
                        row: addr.row,
                        col: addr.col - 1,
                    },
                    cell,
                );
            }
        }
        self.cells = new_cells;
        self.columns -= 1;
    }

    /// 清空指定区域内的所有单元格，不改变行列数。
    pub fn clear_region(&mut self, spec: ClearSpec) {
        match spec {
            ClearSpec::Row(r) => {
                self.cells.retain(|addr, _| addr.row != r);
            }
            ClearSpec::Column(c) => {
                self.cells.retain(|addr, _| addr.col != c);
            }
            ClearSpec::Rect { c1, r1, c2, r2 } => {
                self.cells.retain(|addr, _| {
                    addr.col < c1 || addr.col > c2 || addr.row < r1 || addr.row > r2
                });
            }
        }
    }
}
