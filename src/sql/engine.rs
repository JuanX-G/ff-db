use crate::{sql::{Operator, ast::{ASTNode, ASTRootWrapper, Expr, Literal, Statement}}};
use crate::database::database::*;

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
    fn run_binary_expr(&self, left: &Expr, right: &Expr, op: &Operator) -> Result<SqlExpr, ()> {
        let mut left_val: Literal = Literal::Number(0);
        let mut right_val: Literal = Literal::Number(0);
        match op {
            Operator::Equal => { 
                match &left {
                    Expr::Binary {left: l, op: o, right: r} => {self.run_binary_expr(l, r, o)?;},
                    Expr::Literal(_) => panic!("invalid use of literal"),
                    Expr::Identifier(i) => {
                        let where_cond = self.parse_where(i, &op, match right {
                            Expr::Identifier(_) => panic!("invalid value in where, expcected 'literal'"),
                            Expr::Binary {left: _, op: _, right: _} => panic!("invalid value in where, expected 'literal'"),
                            Expr::Literal(l) => l.clone(),
                        });
                        return Ok(SqlExpr::WhereCondition(where_cond))
                    },
                }
            },
            Operator::NotEqual => { 
                match &left {
                    Expr::Binary {left: l, op: o, right: r} => {self.run_binary_expr(l, r, o)?;},
                    Expr::Literal(_) => panic!("invalid use of literal"),
                    Expr::Identifier(i) => {
                        let where_cond = self.parse_where(i, &op, match right {
                            Expr::Identifier(_) => panic!("invalid value in where, expcected 'literal'"),
                            Expr::Binary {left: _, op: _, right: _} => panic!("invalid value in where, expected 'literal'"),
                            Expr::Literal(l) => l.clone(),
                        });
                        return Ok(SqlExpr::WhereCondition(where_cond))
                    },
                }
            },
            Operator::Greater => { 
                match &left {
                    Expr::Binary {left: l, op: o, right: r} => {self.run_binary_expr(l, r, o)?;},
                    Expr::Literal(_) => panic!("invalid use of literal"),
                    Expr::Identifier(i) => {
                        let where_cond = self.parse_where(i, &op, match right {
                            Expr::Identifier(_) => panic!("invalid value in where, expcected 'literal'"),
                            Expr::Binary {left: _, op: _, right: _} => panic!("invalid value in where, expected 'literal'"),
                            Expr::Literal(l) => l.clone(),
                        });
                        return Ok(SqlExpr::WhereCondition(where_cond))
                    },
                }
            },
            Operator::Smaller => { 
                match &left {
                    Expr::Binary {left: l, op: o, right: r} => {self.run_binary_expr(l, r, o)?;},
                    Expr::Literal(_) => panic!("invalid use of literal"),
                    Expr::Identifier(i) => {
                        let where_cond = self.parse_where(i, op, match right {
                            Expr::Identifier(_) => panic!("invalid value in where, expcected 'literal'"),
                            Expr::Binary {left: _, op: _, right: _} => panic!("invalid value in where, expected 'literal'"),
                            Expr::Literal(l) => l.clone(),
                        });
                        return Ok(SqlExpr::WhereCondition(where_cond))
                    },
                }
            },
            Operator::Plus => {panic!("invalid use of '+' in where clause!")}
        };
        match left {
            Expr::Binary {left: l, op: o, right: r} => {self.run_binary_expr(l, r, o)?;},
            Expr::Literal(l) => left_val = l.clone(),
            Expr::Identifier(_) => panic!("TODO: resolve identifiers"),
        };
        match right {
            Expr::Binary {left: l, op: o, right: r} => {self.run_binary_expr(l, r, o)?;},
            Expr::Literal(l) => right_val = l.clone(),
            Expr::Identifier(_) => panic!("TODO: resolve identifiers"),
        };
        let mut val_n: i32 = 0;
        let mut val_s: String = "".to_string();
        let mut string: bool = false;
        match op {
            Operator::Equal => panic!("TODO: resolve onto a name"),
            Operator::NotEqual => panic!("invalid use of not equal"),
            Operator::Plus => match left_val {
                Literal::Number(nl) => match right_val {
                    Literal::Number(nr) => val_n = nr + nl,
                    Literal::String(_) => panic!("Сannot add a Number and String"),
                },
                Literal::String(sl) => match right_val {
                    Literal::Number(_) => panic!("Сannot add a String and Number"),
                    Literal::String(sr) => {string = true; val_s.push_str(&sl); val_s.push_str(&sr)},
                },
            }
            _ => panic!("invalid use of operator")
        };
        if string {
            Ok(SqlExpr::Literal(Literal::String(val_s)))
        } else {
            Ok(SqlExpr::Literal(Literal::Number(val_n)))
        }
    }
    pub fn run(&self, db: &mut DB) -> Result<QueryResult, Box<dyn std::error::Error>> {
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
                    Some(clauses) => {
                        clauses.iter().map(|cls| { 
                            let res_expr = match cls {
                                Expr::Binary{left: l, op, right: r} => self.run_binary_expr(l, r, op),
                                Expr::Literal(_) => panic!("lone literals not premited in where condition"),
                                Expr::Identifier(_) => panic!("lone identifiers not premited in where condition"),
                            };
                            match res_expr {
                                Ok(expr) => match expr {
                                    SqlExpr::WhereCondition(wc) => wc,
                                    SqlExpr::Literal(_) => panic!("unexpected literal instead of condition in where"),
                                },
                                Err(e) => panic!("TODO: add error on error within expr"),
                            }
                        }).collect()
                    },              
                };
                match db.select_where(s.columns.clone(), where_clauses) {
                    Ok(r) => {return Ok(QueryResult::Rows(r))},
                    Err(_) => panic!("unkown error, TODO!"),
                };
            }
        }
    }
}
