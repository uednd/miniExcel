use crate::model::cell::CellValue;

use super::ast::FunctionName;

pub struct FunctionArgs {
    pub values: Vec<f64>,
    pub count: usize,
}

pub fn collect_values(args: &[CellValue], _name: &FunctionName) -> FunctionArgs {
    let mut values = Vec::new();

    for arg in args {
        match arg {
            CellValue::Number(n) => {
                values.push(*n);
            }
            CellValue::Text(t) => {
                if let Ok(n) = t.parse::<f64>() {
                    values.push(n);
                }
            }
            CellValue::Error(_) => {}
            CellValue::Empty => {}
        }
    }

    let count = values.len();
    FunctionArgs { values, count }
}

pub fn evaluate(name: &FunctionName, collected: &FunctionArgs) -> CellValue {
    match name {
        FunctionName::Sum => {
            if collected.values.is_empty() {
                CellValue::Number(0.0)
            } else {
                CellValue::Number(collected.values.iter().sum())
            }
        }
        FunctionName::Average => {
            if collected.values.is_empty() {
                return CellValue::Number(0.0);
            }
            CellValue::Number(collected.values.iter().sum::<f64>() / collected.values.len() as f64)
        }
        FunctionName::Min => {
            if let Some(min) = collected
                .values
                .iter()
                .fold(None::<f64>, |acc, &x| Some(acc.map_or(x, |a| a.min(x))))
            {
                CellValue::Number(min)
            } else {
                CellValue::Number(0.0)
            }
        }
        FunctionName::Max => {
            if let Some(max) = collected
                .values
                .iter()
                .fold(None::<f64>, |acc, &x| Some(acc.map_or(x, |a| a.max(x))))
            {
                CellValue::Number(max)
            } else {
                CellValue::Number(0.0)
            }
        }
        FunctionName::Count => CellValue::Number(collected.count as f64),
    }
}
