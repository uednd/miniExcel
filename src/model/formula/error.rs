use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CellError {
    DivByZero,
    Ref,
    Value,
    Name,
    Circular,
}

impl CellError {
    pub fn display(&self) -> &'static str {
        match self {
            CellError::DivByZero => "#DIV/0!",
            CellError::Ref => "#REF!",
            CellError::Value => "#VALUE!",
            CellError::Name => "#NAME?",
            CellError::Circular => "#CIRC!",
        }
    }
}
