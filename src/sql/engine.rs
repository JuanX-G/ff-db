use crate::{sql::{Operator, ast::{ASTNode, ASTRootWrapper, Expr, Literal, Statement}}};
use crate::database::database::*;
use std::fmt;
use std::error::Error;

#[derive(Debug)]
pub struct Engine {
    pub ast_root: ASTRootWrapper,
}
#[derive(Debug)]
pub enum QueryResult {
    Rows(Vec<Vec<DBField>>),
    Empty,
}

enum SqlExpr {
    Literal(Literal),
}

#[derive(Debug)]
pub enum EngineError {
    InvalidOperatorInWhere(Operator),
    UnexpectedExprExpectedLiteral(Expr),
    UnexpectedExprExpectedIdentifier(Expr),
    UnexpectedExprExpectedExpression(Expr),
    UnexpectedState,
}
impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            EngineError::InvalidOperatorInWhere(op) => {
                let mut out_s: String = "Invalid operator found on the right of a where statment, found '".to_string();
                out_s.push_str(&op.to_string());
                out_s.push_str("' expected '=' or '<' or '>' or '!='");
                out_s
            }
            EngineError::UnexpectedExprExpectedLiteral(_expr) => {
                let out_s: String = "Invalid expression found on the right of a where statment, expected 'literal' found '".to_string();
                // TODO add proper string method on expr
                out_s
            }
            EngineError::UnexpectedExprExpectedIdentifier(_expr) => {
                let out_s: String = "Invalid expression found on the right of a where statment, expected 'identifier' found '".to_string();
                // TODO add proper string method on expr
                out_s
            },
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

fn compare(
    left: &DBField,
    right: &DBField,
    op: &Operator,
) -> Result<bool, EngineError> {
    match (left, right, op) {
        (DBField::Int(a), DBField::Int(b), Operator::Equal) => Ok(a == b),
        (DBField::Int(a), DBField::Int(b), Operator::NotEqual) => Ok(a != b),
        (DBField::Int(a), DBField::Int(b), Operator::Greater) => Ok(a > b),
        (DBField::Int(a), DBField::Int(b), Operator::Smaller) => Ok(a < b),

        (DBField::Text(a), DBField::Text(b), Operator::Equal) => Ok(a == b),
        (DBField::Text(a), DBField::Text(b), Operator::NotEqual) => Ok(a != b),

        _ => Err(EngineError::UnexpectedState),
    }
}

impl Engine {
    fn resolve_identifier(
        &self,
        name: &str,
        row: &[DBField],
        header: &[DBColumn],
    ) -> Result<DBField, EngineError> {
        let idx = header
            .iter()
            .position(|c| c.name == name)
            .ok_or(EngineError::UnexpectedState)?;
        if idx + 1 > row.len() {return Err(EngineError::UnexpectedState)}
        Ok(row[idx].clone())
    }


    pub fn eval_expr(
        &self,
    expr: &Expr,
    row: &[DBField],
    header: &[DBColumn],
    ) -> Result<bool, EngineError> {
        match expr {
            Expr::Binary { left, op, right } => {
                match op {
                    Operator::Equal
                        | Operator::NotEqual
                        | Operator::Greater
                        | Operator::Smaller 
                        | Operator::And
                        | Operator::Or => {
                            let l = self.eval_value(left, row, header)?;
                            let r = self.eval_value(right, row, header)?;

                            Ok(compare(&l, &r, op)?)
                        }
                }
            }

            _ => Err(EngineError::UnexpectedExprExpectedExpression(expr.clone())),
        }
    }

    fn eval_value(
    &self,
    expr: &Expr,
    row: &[DBField],
    header: &[DBColumn],
    ) -> Result<DBField, EngineError> {
        match expr {
            Expr::Literal(l) => Ok(match l {
                Literal::String(s) => DBField::Text(s.clone()),
                Literal::Number(n) => DBField::Int(*n),
            }),

            Expr::Identifier(name) => {
                self.resolve_identifier(name, row, header)
            },

            _ => Err(EngineError::UnexpectedExprExpectedLiteral(expr.clone())),
        }
    }

    pub fn run(&self, db: &mut Table) -> Result<QueryResult, Box<dyn std::error::Error>> {
        let statment = match &self.ast_root.first_node {
            ASTNode::Statment(s) => s,
            _ => panic!("TODO: add error on missing statment"),
        };
        match statment {
            Statement::Insert(i) => {
                let mut field_to_insert: Vec<DBField> = vec![];
                for val in &i.values {
                    match val {
                        Expr::Literal(l) => match l {
                            Literal::String(s) => field_to_insert.push(DBField::Text(s.clone())),
                            Literal::Number(n) => field_to_insert.push(DBField::Int(*n)),
                        },

                        _ => {
                            return Err(Box::new(
                                    EngineError::UnexpectedExprExpectedLiteral(val.clone())
                            ));
                        }
                    }
                }

                match &i.columns {
                    Some(s) => db.insert(Option::Some(s.iter().map(|cs| cs.as_str()).collect()), field_to_insert)?,
                    None => db.insert(Option::None, field_to_insert)?,
                }
                return Ok(QueryResult::Empty)
            },
            Statement::Select(s) => {
                match &s.where_clause {
                    None => {
                        let r = db.select_cols(
                            s.columns.iter().map(|c| c.as_str()).collect()
                        );
                        Ok(QueryResult::Rows(r?))
                    }
                    Some(where_exprs) => {
                        let r = db.select_where(
                            s.columns.clone(),
                            where_exprs,
                            self,
                        )?;
                        Ok(QueryResult::Rows(r))
                    }
                }
            }

        }
    }
}
