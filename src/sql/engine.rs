use std::error::Error;
use crate::{DB, DBField, sql::{Operator, ast::{ASTNode, ASTRootWrapper, Expr, Literal, Statement}}};
pub struct Engine {
    pub ast_root: ASTRootWrapper,
}
#[derive(Debug)]
pub enum QueryResult {
    Rows(Vec<Vec<DBField>>),
    Empty,
}


/*      ## TODO for run_expr ##
 *      When op == equals
 *          reesolve a columns name table, get columsn names, and return a where clause struct
 * */

impl Engine {
    fn run_expr(&self, left: Expr, right: Expr, op: Operator) -> Result<Literal, ()> {
        let mut left_val: Literal = Literal::Number(0);
        let mut right_val: Literal = Literal::Number(0);
        match left {
            Expr::Binary {left: ref l, op: ref o, right: ref r} => {self.run_expr(*l.clone(), *r.clone(), o.clone())?;},
            Expr::Literal(l) => left_val = l.clone(),
            Expr::Identifier(_) => panic!("TODO: resolve identifiers"),
        };
        match right {
            Expr::Binary {left: ref l, op: o, right: ref r} => {self.run_expr(*l.clone(), *r.clone(), o)?;},
            Expr::Literal(ref l) => right_val = l.clone(),
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
        };
        if string {
            Ok(Literal::String(val_s))
        } else {
            Ok(Literal::Number(val_n))
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
                for val in i.values.clone() {
                    match val {
                        Expr::Literal(l) => match l {
                            Literal::String(s) => field_to_insert.push(DBField::Text(s)),
                            Literal::Number(n) => field_to_insert.push(DBField::Int(n)),
                        },
                        Expr::Binary{left: ref l, op: ref o, right: r} => {
                            match self.run_expr(*l.clone(), *r.clone(), o.clone()) {
                                Ok(l) => match l {
                                    Literal::String(s) => field_to_insert.push(DBField::Text(s)),
                                    Literal::Number(n) => field_to_insert.push(DBField::Int(n)),
                                },
                                Err(_) => panic!("Expr parsing error"),
                            };
                        },
                        Expr::Identifier(_) => panic!("TODO: resolve identifiers")
                    }
                }

                db.insert(i.columns.clone(), field_to_insert)?;
                return Ok(QueryResult::Empty)
            },
            Statement::Select(s) => {
                match s.where_clause {
                    None => match db.select_cols(s.columns.clone()) {
                        Ok(r) => return Ok(QueryResult::Rows(r)),
                        Err(_) => panic!("unkown error"),
                    },
                    Some(clause) => {}
                }
            }
        }
    }

}
