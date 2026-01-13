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


#[derive(Debug)]
pub enum WhereOperator {
    Equal,
    Greater,
    Smaller,
    NotEqual,
}

#[derive(Debug)]
pub struct WhereClause {
    pub col_name: String,
    pub operator: WhereOperator,
    pub expected_value: Literal,
}

enum SqlExpr {
    WhereCondition(WhereClause),
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
                let mut out_s: String = "Invalid expression found on the right of a where statment, expected 'literal' found '".to_string();
                // TODO add proper string method on expr
                out_s
            }
            EngineError::UnexpectedExprExpectedIdentifier(_expr) => {
                let mut out_s: String = "Invalid expression found on the right of a where statment, expected 'identifier' found '".to_string();
                // TODO add proper string method on expr
                out_s
            },
            EngineError::UnexpectedExprExpectedExpression(_expr) => {
                let mut out_s: String = "Invalid expression found in a where statment, expected 'Expression' found '".to_string();
                // TODO add proper string method on expr
                out_s
            },
            EngineError::UnexpectedState => "unexpected state encoutered".to_string(),
        })
    }
}
impl Error for EngineError {}

impl Engine {
    fn parse_where(&self, ident: &str, op: &Operator, expected_val: Literal) -> WhereClause {
        let clause_op = match op {
            Operator::Equal => WhereOperator::Equal,
            Operator::NotEqual => WhereOperator::NotEqual,
            Operator::Smaller => WhereOperator::Smaller,
            Operator::Greater => WhereOperator::Greater,
            _ => panic!("invalid operator in where"),
        };
        WhereClause{col_name: ident.to_string(), operator: clause_op, expected_value: expected_val}
    }
    fn parse_where_condition(&self, ident: &str, op: &Operator, expected_val: &Expr) 
        -> Result<WhereClause, Box<dyn std::error::Error>> {
            let expected_val = match expected_val {
                Expr::Identifier(_) => return Err(Box::new(EngineError::UnexpectedExprExpectedLiteral(expected_val.clone()))),
                Expr::Binary {left: _, op: _, right: _} => return Err(Box::new(EngineError::UnexpectedExprExpectedLiteral(expected_val.clone()))),
                Expr::Literal(l) => l.clone(),
            };
            Ok(self.parse_where(ident, op, expected_val))

    }
    fn run_binary_expr(&self, left: &Expr, right: &Expr, op: &Operator) -> Result<SqlExpr, Box<dyn std::error::Error>> {
        let mut left_val: Literal = Literal::Number(0);
        let mut right_val: Literal = Literal::Number(0);
        match op {
            Operator::Equal => { 
                match &left {
                    Expr::Binary {left: l, op: o, right: r} => {self.run_binary_expr(l, r, o)?;},
                    Expr::Literal(_) => return Err(Box::new(EngineError::UnexpectedExprExpectedIdentifier(left.clone()))),
                    Expr::Identifier(i) => {
                        let where_cond = self.parse_where_condition(i, &op, right)?;
                        return Ok(SqlExpr::WhereCondition(where_cond))
                    },
                }
            },
            Operator::NotEqual => { 
                match &left {
                    Expr::Binary {left: l, op: o, right: r} => {self.run_binary_expr(l, r, o)?;},
                    Expr::Literal(_) => return Err(Box::new(EngineError::UnexpectedExprExpectedIdentifier(left.clone()))), 
                    Expr::Identifier(i) => {
                        let where_cond = self.parse_where(i, &op, match right {
                            Expr::Identifier(_) => return Err(Box::new(EngineError::UnexpectedExprExpectedLiteral(left.clone()))),
                            Expr::Binary {left: _, op: _, right: _} => return Err(Box::new(EngineError::UnexpectedExprExpectedLiteral(left.clone()))),
                            Expr::Literal(l) => l.clone(),
                        });
                        return Ok(SqlExpr::WhereCondition(where_cond))
                    },
                }
            },
            Operator::Greater => { 
                match &left {
                    Expr::Binary {left: l, op: o, right: r} => {self.run_binary_expr(l, r, o)?;},
                    Expr::Literal(_) => return Err(Box::new(EngineError::UnexpectedExprExpectedLiteral(left.clone()))),
                    Expr::Identifier(i) => {
                        let where_cond = self.parse_where_condition(i, &op, right)?;
                        return Ok(SqlExpr::WhereCondition(where_cond))
                    },
                }
            },
            Operator::Smaller => { 
                match &left {
                    Expr::Binary {left: l, op: o, right: r} => {self.run_binary_expr(l, r, o)?;},
                    Expr::Literal(_) => return Err(Box::new(EngineError::UnexpectedExprExpectedIdentifier(left.clone()))),
                    Expr::Identifier(i) => {
                        let where_cond = self.parse_where_condition(i, &op, right)?;
                        return Ok(SqlExpr::WhereCondition(where_cond))
                    },
                }
            },
            Operator::Plus => return Err(Box::new(EngineError::InvalidOperatorInWhere(op.clone()))),
        };
        Err(Box::new(EngineError::UnexpectedState))
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
                            Literal::String(s) => field_to_insert.push(DBField::Text(s.to_string())),
                            Literal::Number(n) => field_to_insert.push(DBField::Int(*n)),
                        },
                        Expr::Binary{left: l, op: o, right: r} => {
                            match self.run_binary_expr(l, r, o) {
                                Ok(ex) => match ex {
                                    SqlExpr::Literal(l) => match l {
                                        Literal::String(s) => field_to_insert.push(DBField::Text(s)),
                                        Literal::Number(n) => field_to_insert.push(DBField::Int(n)),
                                    },
                                    SqlExpr::WhereCondition(_) => panic!("unexpected where in isert statment"),
                                },
                                Err(_) => panic!("Expr parsing error"),
                            };
                        },
                        Expr::Identifier(_) => panic!("TODO: resolve identifiers")
                    }
                }
                match &i.columns {
                    Some(s) => db.insert(Option::Some(s.iter().map(|cs| cs.as_str()).collect()), field_to_insert)?,
                    None => db.insert(Option::None, field_to_insert)?,
                }
                return Ok(QueryResult::Empty)
            },
            Statement::Select(s) => {
                dbg!(&s);
                let where_clauses = match &s.where_clause {
                    None => match db.select_cols(s.columns.iter().map(|cs| cs.as_str()).collect()) {
                        Ok(r) => return Ok(QueryResult::Rows(r)),
                        Err(_) => panic!("unkown error, TODO!"),
                    },
                    Some(clauses) =>  {
                        let where_clauses = clauses
                            .iter()
                            .map(|cls| -> Result<WhereClause, Box<dyn std::error::Error>> {
                                let expr = match cls {
                                    Expr::Binary { left: l, op, right: r } =>
                                        self.run_binary_expr(l, r, op)?,

                                    Expr::Literal(_) =>
                                        return Err(Box::new(
                                                EngineError::UnexpectedExprExpectedExpression(
                                                    Expr::Literal(Literal::String("aa".to_string()))
                                                )
                                        )),

                                    Expr::Identifier(_) =>
                                        panic!("lone identifiers not permitted in where condition"),
                                };

                                match expr {
                                    SqlExpr::WhereCondition(wc) => Ok(wc),
                                    SqlExpr::Literal(_) =>
                                        panic!("unexpected literal instead of condition in where"),
                                }
                            })
                        .collect::<Result<Vec<WhereClause>, Box<dyn std::error::Error>>>()?;
                        where_clauses
                    }
                };
                match db.select_where(s.columns.clone(), where_clauses) {
                    Ok(r) => {return Ok(QueryResult::Rows(r))},
                    Err(e) => return Err(e),
                };
            }
        }
    }
}
