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
    let reader = WorkbookReader {
        cells: &wb.cells,
    };
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::workbook::ClearSpec;

    fn a(row: usize, col: usize) -> CellAddress {
        CellAddress { row, col }
    }

    #[test]
    fn test_workbook_set_text_formula() {
        let mut wb = Workbook::new("test".to_string(), 10, 10);
        wb.set_text(a(0, 0), "=1+2".to_string());

        let cell = wb.get_cell(a(0, 0)).unwrap();
        assert_eq!(cell.raw, "=1+2");
        assert_eq!(cell.value, CellValue::Number(3.0));
    }

    #[test]
    fn test_workbook_formula_cell_ref() {
        let mut wb = Workbook::new("test".to_string(), 10, 10);
        wb.set_text(a(0, 1), "42".to_string());
        wb.set_text(a(0, 0), "=B1+8".to_string());

        let cell = wb.get_cell(a(0, 0)).unwrap();
        assert_eq!(cell.value, CellValue::Number(50.0));
    }

    #[test]
    fn test_workbook_formula_update_dependency() {
        let mut wb = Workbook::new("test".to_string(), 10, 10);
        wb.set_text(a(0, 1), "5".to_string());
        wb.set_text(a(0, 0), "=B1*2".to_string());
        assert_eq!(wb.get_cell(a(0, 0)).unwrap().value, CellValue::Number(10.0));

        wb.set_text(a(0, 1), "7".to_string());
        assert_eq!(wb.get_cell(a(0, 1)).unwrap().value, CellValue::Text("7".to_string()));

        wb.recalc();
        assert_eq!(wb.get_cell(a(0, 0)).unwrap().value, CellValue::Number(14.0));
    }

    #[test]
    fn test_workbook_clear_region_triggers_recalc() {
        let mut wb = Workbook::new("test".to_string(), 10, 10);
        wb.set_text(a(0, 1), "=3+4".to_string());
        wb.set_text(a(0, 0), "=B1+1".to_string());
        assert_eq!(wb.get_cell(a(0, 0)).unwrap().value, CellValue::Number(8.0));

        wb.clear_region(ClearSpec::Rect {
            c1: 1,
            r1: 0,
            c2: 1,
            r2: 0,
        });
        assert!(wb.get_cell(a(0, 1)).is_none());
        assert_eq!(wb.get_cell(a(0, 0)).unwrap().value, CellValue::Number(1.0));
    }

    #[test]
    fn test_workbook_non_formula_preserved() {
        let mut wb = Workbook::new("test".to_string(), 10, 10);
        wb.set_text(a(0, 0), "hello".to_string());

        let cell = wb.get_cell(a(0, 0)).unwrap();
        assert_eq!(cell.raw, "hello");
        assert_eq!(cell.value, CellValue::Text("hello".to_string()));
    }

    #[test]
    fn test_workbook_div_by_zero() {
        let mut wb = Workbook::new("test".to_string(), 10, 10);
        wb.set_text(a(0, 0), "=1/0".to_string());

        let cell = wb.get_cell(a(0, 0)).unwrap();
        assert_eq!(cell.raw, "=1/0");
        match &cell.value {
            CellValue::Error(e) => assert_eq!(e.display(), "#DIV/0!"),
            _ => panic!("expected error"),
        }
    }

    #[test]
    fn test_workbook_builtin_sum() {
        let mut wb = Workbook::new("test".to_string(), 10, 10);
        wb.set_text(a(0, 0), "10".to_string());
        wb.set_text(a(1, 0), "20".to_string());
        wb.set_text(a(2, 0), "30".to_string());
        wb.set_text(a(3, 0), "=SUM(A1:A3)".to_string());
        assert_eq!(
            wb.get_cell(a(3, 0)).unwrap().value,
            CellValue::Number(60.0)
        );
    }

    #[test]
    fn test_workbook_average() {
        let mut wb = Workbook::new("test".to_string(), 10, 10);
        wb.set_text(a(0, 0), "10".to_string());
        wb.set_text(a(1, 0), "20".to_string());
        wb.set_text(a(2, 0), "=AVERAGE(A1:A2)".to_string());
        assert_eq!(
            wb.get_cell(a(2, 0)).unwrap().value,
            CellValue::Number(15.0)
        );
    }
}
