use crate::model::cell::CellAddress;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(f64),
    CellRef(CellAddress),
    Range(CellAddress, CellAddress),
    BinaryOp(Box<Expr>, BinaryOp, Box<Expr>),
    UnaryOp(UnaryOp, Box<Expr>),
    Function(FunctionName, Vec<Expr>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOp {
    Neg,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionName {
    Sum,
    Average,
    Min,
    Max,
    Count,
}
