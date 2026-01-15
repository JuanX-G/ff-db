use crate::sql::{Operator, ast::{ASTNode, ASTRootWrapper, Expr, Literal, Statement}};
use crate::database::{DBColumn, DBField, table::Table};
use crate::sql::errors::EngineError;

#[derive(Debug)]
pub enum QueryResult {
    Rows(Vec<Vec<DBField>>),
    Empty,
}

pub type EngineResult<T> = Result<T, EngineError>;

///
/// # compares two fields according to the 'op' operator 
///
/// # Errors
///
/// EngineError on issues
pub fn compare(
    left: &DBField,
    right: &DBField,
    op: &Operator,
    ) -> EngineResult<bool> {
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

///
/// # The struct for evaluating an ast
///
/// ast_root wraps the first statment of sql, that is to be walked and run
///
/// # Errors
///
/// Methods, in general, return EngineError on failure
#[derive(Debug)]
pub struct Engine {
    pub ast_root: ASTRootWrapper,
}

impl Engine {
    /// resolves, from a provided header, the coresponding DBField
    ///
    /// # Errors
    ///
    /// Returns EngineError wrraped in a Result, uses the Ok variant on succes
    /// The EngineError being described in the errors sub module
    fn resolve_identifier(
        &self,
        name: &str,
        row: &[DBField],
        header: &[DBColumn],
    ) -> EngineResult<DBField> {
        let idx = header
            .iter()
            .position(|c| c.name == name)
            .ok_or(EngineError::UnexpectedState)?;
        if idx + 1 > row.len() {return Err(EngineError::UnexpectedState)}
        Ok(row[idx].clone())
    }

    /// Evaluates a logical expression
    ///
    /// # Errors
    ///
    /// Returns EngineError wrraped in a Result, uses the Ok variant on succes
    /// The EngineError being described in the errors sub module
    pub fn eval_expr(
        &self,
        expr: &Expr,
        row: &[DBField],
        header: &[DBColumn],
    ) -> EngineResult<bool> {
        match expr {
            Expr::Binary { left, op, right } => {
                match op {
                    Operator::Equal
                        | Operator::NotEqual
                        | Operator::Greater
                        | Operator::Smaller => {
                            let l = self.eval_value(left, row, header)?;
                            let r = self.eval_value(right, row, header)?;

                            Ok(compare(&l, &r, op)?)
                        }
                    Operator::And => {
                        Ok(
                            self.eval_expr(left, row, header)? &&
                            self.eval_expr(right, row, header)?
                        )
                    }
                    Operator::Or => {
                        Ok(
                            self.eval_expr(left, row, header)? ||
                            self.eval_expr(right, row, header)?
                        )
                    }
                }
            }
            _ => Err(EngineError::UnexpectedExprExpectedExpression(expr.clone())),
        }
    }

    /// Evaluates a value --- a name or a literal 
    ///
    /// # Errors
    ///
    /// Returns EngineError wrraped in a Result, uses the Ok variant on succes
    /// The EngineError being described in the errors sub module
    fn eval_value(
    &self,
    expr: &Expr,
    row: &[DBField],
    header: &[DBColumn],
    ) -> EngineResult<DBField> {
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

    /// Evaluates the AST
    ///
    /// # Errors
    ///
    /// Returns a boxed error, usually an EngineError. The EngineError being 
    /// described the errors sub-module.
    /// also possible are database based errors.
    pub fn run(&self, db: &mut Table) -> Result<QueryResult, Box<dyn std::error::Error>> {
        let statment = match &self.ast_root.first_node {
            ASTNode::Statment(s) => s,
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
