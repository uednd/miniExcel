pub mod ast;
pub mod error;
pub mod eval;
pub mod functions;
pub mod parser;

use std::collections::HashMap;

use crate::model::{
    cell::{Cell, CellAddress, CellValue},
    workbook::Workbook,
};

use self::eval::{CellReader, Evaluator};

struct WorkbookReader<'a> {
    cells: &'a HashMap<CellAddress, Cell>,
}

impl CellReader for WorkbookReader<'_> {
    fn get_raw_and_value(&self, addr: CellAddress) -> Option<(&str, &CellValue)> {
        self.cells
            .get(&addr)
            .map(|cell| (cell.raw.as_str(), &cell.value))
    }
}

pub fn recalc(wb: &mut Workbook) {
    let reader = WorkbookReader { cells: &wb.cells };
    let mut evaluator = Evaluator::new(&reader);

    let addrs: Vec<CellAddress> = wb.cells.keys().copied().collect();
    let mut updates: Vec<(CellAddress, CellValue)> = Vec::new();

    for addr in addrs {
        if let Some(cell) = wb.cells.get(&addr) {
            if cell.raw.starts_with('=') {
                let result = evaluator.eval_cell(addr);
                let new_val = result.unwrap_or_else(|e| CellValue::Error(e));
                updates.push((addr, new_val));
            }
        }
    }

    drop(evaluator);
    drop(reader);

    for (addr, value) in updates {
        if let Some(cell) = wb.cells.get_mut(&addr) {
            cell.value = value;
        }
    }
}
