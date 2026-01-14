use crate::sql::*;
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Insert(InsertStatement),
    Select(SelectStatement),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    String(String),
    Number(i32),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Binary {
        left: Box<Expr>,
        op: Operator,
        right: Box<Expr>,
    },
    Identifier(String),
    Literal(Literal),
}

#[derive(Debug, Clone, PartialEq)]
pub struct InsertStatement {
    pub table: String,
    pub columns: Option<Vec<String>>,
    pub values: Vec<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SelectStatement {
    pub columns: Vec<String>,
    pub table: String,
    pub where_clause: Option<Vec<Expr>>,
}
#[derive(Debug)]
pub struct ASTRootWrapper {
    pub first_node: ASTNode,
}

#[derive(Debug)]
pub enum ASTNode {
    Statment(Statement),
}


