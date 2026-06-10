use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

use super::cell::{Cell, CellAddress, CellValue};
use super::limits;

/// 清空单元格内容的范围。
///
/// 行列索引均从 0 开始。`Rect` 的四个端点必须已经归一化：
/// `c1 <= c2` 且 `r1 <= r2`。
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

/// 工作簿数据模型。
///
/// `columns` 和 `rows` 表示当前表格尺寸，`cells` 只保存有内容的单元格。
/// 调用者应保证传入的 `CellAddress` 位于当前尺寸内。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workbook {
    pub version: u8,
    pub name: String,
    pub columns: usize,
    pub rows: usize,
    pub cells: HashMap<CellAddress, Cell>,
}

impl Workbook {
    /// 创建空工作簿。
    ///
    /// `columns` 和 `rows` 应至少为 1。
    pub fn new(name: String, columns: usize, rows: usize) -> Self {
        Self {
            version: 1,
            name,
            columns,
            rows,
            cells: HashMap::new(),
        }
    }

    /// 将工作簿以 JSON 写入指定路径。
    ///
    /// 序列化失败或文件写入失败时返回 `io::Error`。
    pub fn save<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)
    }

    /// 从指定路径读取工作簿，并在加载后执行公式重算。
    ///
    /// 文件读取失败或 JSON 解析失败时返回 `io::Error`。
    pub fn load<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let raw = fs::read_to_string(path)?;
        let mut wb: Self = serde_json::from_str(&raw)?;
        wb.recalc();
        Ok(wb)
    }

    /// 读取指定地址的单元格。
    ///
    /// 空白单元格不会出现在 `cells` 中，因此返回 `None`。
    pub fn get_cell(&self, addr: CellAddress) -> Option<&Cell> {
        self.cells.get(&addr)
    }

    /// 写入指定地址的文本。
    ///
    /// 该方法不检查地址是否越界，调用者负责传入合法地址。
    /// 空文本会清除单元格。
    /// 以 `=` 开头的文本会被识别为公式并触发重算。
    pub fn set_text(&mut self, addr: CellAddress, raw: String) {
        if raw.is_empty() {
            self.clear_cell(addr);
            return;
        }

        let is_formula = raw.starts_with('=');
        let value = if is_formula {
            CellValue::Empty
        } else {
            CellValue::Text(raw.clone())
        };

        self.cells.insert(addr, Cell { raw, value });

        if is_formula {
            self.recalc();
        }
    }

    /// 重算所有公式单元格。
    pub fn recalc(&mut self) {
        super::formula::recalc(self);
    }

    /// 清除指定地址的单元格内容。
    pub fn clear_cell(&mut self, addr: CellAddress) {
        self.cells.remove(&addr);
    }

    /// 确保表格行列数不小于指定值。
    ///
    /// 粘贴时自动扩展表格。不会超过最大行列限制。
    pub fn ensure_size(&mut self, rows: usize, cols: usize) {
        self.rows = self.rows.max(rows).min(limits::MAX_ROWS);
        self.columns = self.columns.max(cols).min(limits::MAX_COLUMNS);
    }

    /// 删除指定行，该行上方的行不动，下方行上移，总行数减一。
    ///
    /// 至少保留一行。该方法只维护工作簿尺寸和单元格位置；
    /// 光标、滚动和选区由编辑器状态负责同步。
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
        self.recalc();
    }

    /// 删除指定列，该列左侧的列不动，右侧列左移，总列数减一。
    ///
    /// 至少保留一列。该方法只维护工作簿尺寸和单元格位置；
    /// 光标、滚动和选区由编辑器状态负责同步。
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
        self.recalc();
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
        self.recalc();
    }
}
