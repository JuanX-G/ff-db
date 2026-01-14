use std::{fmt, error::Error};
use crate::sql::{ast::Expr};

#[derive(Debug)]
pub enum EngineError {
    UnexpectedExprExpectedLiteral(Expr),
    UnexpectedExprExpectedExpression(Expr),
    UnexpectedState,
}
impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            EngineError::UnexpectedExprExpectedLiteral(_expr) => {
                let out_s: String = "Invalid expression found on the right of a where statment, expected 'literal' found '".to_string();
                // TODO add proper string method on expr
                out_s
            }
            EngineError::UnexpectedExprExpectedExpression(_expr) => {
                let out_s: String = "Invalid expression found in a where statment, expected 'Expression' found '".to_string();
                // TODO add proper string method on expr
                out_s
            },
            EngineError::UnexpectedState => "unexpected state encoutered".to_string(),
        })
    }
}
impl Error for EngineError {}
