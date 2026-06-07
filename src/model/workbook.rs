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
}

/// Tuple 转 String 作为 HashMap 的 key，用于 JSON 序列化和反序列化 
fn coord_key(coord: Coord) -> String {
    format!("{},{}", coord.0, coord.1)
}
