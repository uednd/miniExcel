use std::collections::{HashMap, HashSet};

use crate::model::cell::{CellAddress, CellValue};

use super::{
    ast::{BinaryOp, Expr, UnaryOp},
    error::CellError,
    functions, parser,
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
            return match val {
                CellValue::Error(e) => Err(e.clone()),
                _ => Ok(val.clone()),
            };
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

        if let Some(formula) = raw.strip_prefix('=') {
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
