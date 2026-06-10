use pest::Parser;
use pest_derive::Parser as PestParser;

use crate::model::cell::CellAddress;

use super::ast::{BinaryOp, Expr, FunctionName, UnaryOp};

#[derive(PestParser)]
#[grammar = "model/formula/formula.pest"]
struct FormulaParser;

pub fn parse(input: &str) -> Result<Expr, String> {
    let pairs = FormulaParser::parse(Rule::formula, input).map_err(|e| e.to_string())?;
    let pair = pairs.into_iter().next().ok_or("empty formula")?;
    Ok(build_expr(pair))
}

fn build_expr(pair: pest::iterators::Pair<Rule>) -> Expr {
    match pair.as_rule() {
        Rule::number => Expr::Number(pair.as_str().parse().unwrap()),
        Rule::cell_ref => parse_cell_ref(pair.as_str()),
        Rule::range => {
            let mut inner = pair.into_inner();
            let start = parse_cell_address(inner.next().unwrap().as_str());
            let end = parse_cell_address(inner.next().unwrap().as_str());
            Expr::Range(start, end)
        }
        Rule::function_call => {
            let mut inner = pair.into_inner();
            let name_pair = inner.next().unwrap();
            let args_pair = inner.next().unwrap();
            let func = match name_pair.as_str() {
                "SUM" => FunctionName::Sum,
                "AVERAGE" => FunctionName::Average,
                "MIN" => FunctionName::Min,
                "MAX" => FunctionName::Max,
                "COUNT" => FunctionName::Count,
                _ => unreachable!(),
            };
            let args = build_arg_list(args_pair);
            Expr::Function(func, args)
        }
        Rule::arg_list => unreachable!(),
        Rule::unary => {
            let s = pair.as_str();
            let child = pair.into_inner().next().unwrap();
            if s.starts_with('-') {
                Expr::UnaryOp(UnaryOp::Neg, Box::new(build_expr(child)))
            } else {
                build_expr(child)
            }
        }
        Rule::factor => build_binary_chain(pair),
        Rule::term => build_binary_chain(pair),
        Rule::formula => {
            let inner = pair.into_inner().next().unwrap();
            build_expr(inner)
        }
        Rule::add_op | Rule::mul_op => unreachable!(),
        _ => unreachable!(),
    }
}

fn build_binary_chain(pair: pest::iterators::Pair<Rule>) -> Expr {
    let mut inner = pair.into_inner();
    let mut left = build_expr(inner.next().unwrap());
    while let Some(op_pair) = inner.next() {
        let right = build_expr(inner.next().unwrap());
        let op = match op_pair.as_str() {
            "*" => BinaryOp::Mul,
            "/" => BinaryOp::Div,
            "+" => BinaryOp::Add,
            "-" => BinaryOp::Sub,
            _ => unreachable!(),
        };
        left = Expr::BinaryOp(Box::new(left), op, Box::new(right));
    }
    left
}

fn build_arg_list(pair: pest::iterators::Pair<Rule>) -> Vec<Expr> {
    pair.into_inner().map(build_expr).collect()
}

fn parse_cell_ref(s: &str) -> Expr {
    let addr = parse_cell_address(s);
    Expr::CellRef(addr)
}

fn parse_cell_address(s: &str) -> CellAddress {
    let digit_pos = s.find(|c: char| c.is_ascii_digit()).unwrap();
    let col_str = &s[..digit_pos];
    let row_str = &s[digit_pos..];
    let col = col_name_to_index(col_str);
    let row: usize = row_str.parse().unwrap();
    CellAddress {
        row: row.saturating_sub(1),
        col,
    }
}

fn col_name_to_index(s: &str) -> usize {
    let mut result = 0usize;
    for c in s.chars() {
        let index = (c.to_ascii_uppercase() as usize - 'A' as usize) + 1;
        result = result * 26 + index;
    }
    result.saturating_sub(1)
}

