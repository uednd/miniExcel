use std::collections::{HashMap, HashSet};

use crate::model::cell::{CellAddress, CellValue};

use super::{
    ast::{BinaryOp, Expr, UnaryOp},
    error::CellError,
    functions,
    parser,
};

pub trait CellReader {
    fn get_raw_and_value(&self, addr: CellAddress) -> Option<(&str, &CellValue)>;
}

pub struct Evaluator<'a> {
    reader: &'a dyn CellReader,
    memo: HashMap<CellAddress, CellValue>,
    evaluating: HashSet<CellAddress>,
}

impl<'a> Evaluator<'a> {
    pub fn new(reader: &'a dyn CellReader) -> Self {
        Self {
            reader,
            memo: HashMap::new(),
            evaluating: HashSet::new(),
        }
    }

    pub fn eval_cell(&mut self, addr: CellAddress) -> Result<CellValue, CellError> {
        if let Some(val) = self.memo.get(&addr) {
            return Ok(val.clone());
        }

        if self.evaluating.contains(&addr) {
            self.memo
                .insert(addr, CellValue::Error(CellError::Circular));
            return Err(CellError::Circular);
        }

        let (raw, value) = match self.reader.get_raw_and_value(addr) {
            Some(tuple) => tuple,
            None => return Ok(CellValue::Empty),
        };

        if raw.starts_with('=') {
            let formula = &raw[1..];
            self.evaluating.insert(addr);
            let ast = parser::parse(formula.trim()).map_err(|_| CellError::Value)?;
            let result = self.eval_expr(&ast, addr);
            self.evaluating.remove(&addr);
            match result {
                Ok(val) => {
                    self.memo.insert(addr, val.clone());
                    Ok(val)
                }
                Err(e) => {
                    self.memo.insert(addr, CellValue::Error(e.clone()));
                    Err(e)
                }
            }
        } else {
            Ok(value.clone())
        }
    }

    fn eval_expr(&mut self, expr: &Expr, origin: CellAddress) -> Result<CellValue, CellError> {
        match expr {
            Expr::Number(n) => Ok(CellValue::Number(*n)),
            Expr::CellRef(addr) => self.eval_cell(*addr),
            Expr::Range(_start, _end) => Err(CellError::Value),
            Expr::BinaryOp(left, op, right) => {
                let lv = self.eval_expr(left, origin)?;
                let rv = self.eval_expr(right, origin)?;
                apply_binary_op(&lv, op, &rv)
            }
            Expr::UnaryOp(UnaryOp::Neg, inner) => {
                let v = self.eval_expr(inner, origin)?;
                match v {
                    CellValue::Number(n) => Ok(CellValue::Number(-n)),
                    _ => Err(CellError::Value),
                }
            }
            Expr::Function(name, args) => {
                let mut all_values = Vec::new();
                for arg in args {
                    match arg {
                        Expr::Range(start, end) => {
                            for addr in expand_range(*start, *end, origin) {
                                all_values.push(self.eval_cell(addr)?);
                            }
                        }
                        _ => {
                            all_values.push(self.eval_expr(arg, origin)?);
                        }
                    }
                }
                let collected = functions::collect_values(&all_values, name);
                Ok(functions::evaluate(name, &collected))
            }
        }
    }
}

fn expand_range(start: CellAddress, end: CellAddress, _origin: CellAddress) -> Vec<CellAddress> {
    let row_min = start.row.min(end.row);
    let row_max = start.row.max(end.row);
    let col_min = start.col.min(end.col);
    let col_max = start.col.max(end.col);
    let mut cells = Vec::with_capacity((row_max - row_min + 1) * (col_max - col_min + 1));
    for r in row_min..=row_max {
        for c in col_min..=col_max {
            cells.push(CellAddress { row: r, col: c });
        }
    }
    cells
}

fn apply_binary_op(
    left: &CellValue,
    op: &BinaryOp,
    right: &CellValue,
) -> Result<CellValue, CellError> {
    if let CellValue::Error(e) = left {
        return Err(e.clone());
    }
    if let CellValue::Error(e) = right {
        return Err(e.clone());
    }
    let l = cell_to_f64(left)?;
    let r = cell_to_f64(right)?;
    match op {
        BinaryOp::Add => Ok(CellValue::Number(l + r)),
        BinaryOp::Sub => Ok(CellValue::Number(l - r)),
        BinaryOp::Mul => Ok(CellValue::Number(l * r)),
        BinaryOp::Div => {
            if r == 0.0 {
                Err(CellError::DivByZero)
            } else {
                Ok(CellValue::Number(l / r))
            }
        }
    }
}

fn cell_to_f64(v: &CellValue) -> Result<f64, CellError> {
    match v {
        CellValue::Number(n) => Ok(*n),
        CellValue::Text(t) => t.parse::<f64>().map_err(|_| CellError::Value),
        CellValue::Empty => Ok(0.0),
        CellValue::Error(e) => Err(e.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::cell::CellValue;
    use crate::model::formula::ast::{FunctionName, UnaryOp};

    struct TestReader {
        cells: HashMap<CellAddress, (String, CellValue)>,
    }

    impl TestReader {
        fn new() -> Self {
            Self {
                cells: HashMap::new(),
            }
        }

        fn set(&mut self, row: usize, col: usize, raw: &str, val: CellValue) {
            self.cells
                .insert(CellAddress { row, col }, (raw.to_string(), val));
        }
    }

    impl CellReader for TestReader {
        fn get_raw_and_value(&self, addr: CellAddress) -> Option<(&str, &CellValue)> {
            self.cells.get(&addr).map(|(r, v)| (r.as_str(), v))
        }
    }

    fn addr(row: usize, col: usize) -> CellAddress {
        CellAddress { row, col }
    }

    #[test]
    fn test_arithmetic() {
        let reader = TestReader::new();
        let mut ev = Evaluator::new(&reader);
        assert_eq!(
            ev.eval_cell(addr(0, 0)).unwrap(),
            CellValue::Empty
        );
    }

    #[test]
    fn test_parse_and_eval_static() {
        let reader = TestReader::new();
        let mut ev = Evaluator::new(&reader);
        // evaluate formula text as a CellRef expression
        assert_eq!(
            ev.eval_expr(
                &Expr::BinaryOp(
                    Box::new(Expr::Number(1.0)),
                    BinaryOp::Add,
                    Box::new(Expr::BinaryOp(
                        Box::new(Expr::Number(2.0)),
                        BinaryOp::Mul,
                        Box::new(Expr::Number(3.0)),
                    )),
                ),
                addr(0, 0),
            )
            .unwrap(),
            CellValue::Number(7.0)
        );
    }

    #[test]
    fn test_div_by_zero() {
        let reader = TestReader::new();
        let mut ev = Evaluator::new(&reader);
        assert_eq!(
            ev.eval_expr(
                &Expr::BinaryOp(
                    Box::new(Expr::Number(1.0)),
                    BinaryOp::Div,
                    Box::new(Expr::Number(0.0)),
                ),
                addr(0, 0),
            )
            .unwrap_err(),
            CellError::DivByZero
        );
    }

    #[test]
    fn test_cell_ref() {
        let mut reader = TestReader::new();
        reader.set(0, 1, "10", CellValue::Number(10.0));
        let mut ev = Evaluator::new(&reader);
        assert_eq!(
            ev.eval_cell(addr(0, 1)).unwrap(),
            CellValue::Number(10.0)
        );
    }

    #[test]
    fn test_simple_formula_cell() {
        let mut reader = TestReader::new();
        reader.set(0, 0, "=1+2", CellValue::Empty);
        let mut ev = Evaluator::new(&reader);
        assert_eq!(
            ev.eval_cell(addr(0, 0)).unwrap(),
            CellValue::Number(3.0)
        );
    }

    #[test]
    fn test_referenced_formula() {
        let mut reader = TestReader::new();
        reader.set(0, 0, "=B1+5", CellValue::Empty);
        reader.set(0, 1, "=10", CellValue::Empty);
        let mut ev = Evaluator::new(&reader);
        assert_eq!(
            ev.eval_cell(addr(0, 0)).unwrap(),
            CellValue::Number(15.0)
        );
    }

    #[test]
    fn test_circular_ref() {
        let mut reader = TestReader::new();
        reader.set(0, 0, "=A2", CellValue::Empty);
        reader.set(1, 0, "=A1", CellValue::Empty);
        let mut ev = Evaluator::new(&reader);
        let result = ev.eval_cell(addr(0, 0));
        assert!(matches!(result, Err(CellError::Circular)));
    }

    #[test]
    fn test_self_circular() {
        let mut reader = TestReader::new();
        reader.set(0, 0, "=A1+1", CellValue::Empty);
        let mut ev = Evaluator::new(&reader);
        let result = ev.eval_cell(addr(0, 0));
        assert!(matches!(result, Err(CellError::Circular)));
    }

    #[test]
    fn test_latex_style() {
        let mut reader = TestReader::new();
        reader.set(0, 0, "=SUM(B1:B3)", CellValue::Empty);
        reader.set(0, 1, "1", CellValue::Text("1".to_string()));
        reader.set(1, 1, "2", CellValue::Text("2".to_string()));
        reader.set(2, 1, "3", CellValue::Text("3".to_string()));
        let mut ev = Evaluator::new(&reader);
        assert_eq!(
            ev.eval_cell(addr(0, 0)).unwrap(),
            CellValue::Number(6.0)
        );
    }

    #[test]
    fn test_sum_with_individual_cells() {
        let mut reader = TestReader::new();
        reader.set(0, 0, "=SUM(B1,B2,B3)", CellValue::Empty);
        reader.set(0, 1, "10", CellValue::Number(10.0));
        reader.set(1, 1, "20", CellValue::Number(20.0));
        reader.set(2, 1, "30", CellValue::Number(30.0));
        let mut ev = Evaluator::new(&reader);
        assert_eq!(
            ev.eval_cell(addr(0, 0)).unwrap(),
            CellValue::Number(60.0)
        );
    }

    #[test]
    fn test_sum_with_constant() {
        let reader = TestReader::new();
        let mut ev = Evaluator::new(&reader);
        let expr = Expr::Function(
            FunctionName::Sum,
            vec![Expr::Number(10.0), Expr::Number(20.0)],
        );
        assert_eq!(
            ev.eval_expr(&expr, addr(0, 0)).unwrap(),
            CellValue::Number(30.0)
        );
    }

    #[test]
    fn test_unary_neg() {
        let reader = TestReader::new();
        let mut ev = Evaluator::new(&reader);
        let expr = Expr::BinaryOp(
            Box::new(Expr::UnaryOp(UnaryOp::Neg, Box::new(Expr::Number(5.0)))),
            BinaryOp::Add,
            Box::new(Expr::Number(3.0)),
        );
        assert_eq!(
            ev.eval_expr(&expr, addr(0, 0)).unwrap(),
            CellValue::Number(-2.0)
        );
    }
}
