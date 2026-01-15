pub mod lexer;
pub mod parser;
pub mod ast;
pub mod engine;
pub mod errors;

#[derive(Debug, Clone, PartialEq)]
pub enum SqlToken {
    Keyword(SqlKeyword),
    Identifier(String),
    StringLiteral(String),
    NumberLiteral(String),
    Operator(Operator),
    Comma,
    Semicolon,
    LeftParen,
    RightParen,
    EOF,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SqlKeyword {
    Select,
    From,
    Insert,
    Into,
    Values,
    Where,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    Equal,
    NotEqual,
    Greater,
    Smaller,
    And,
    Or,
}

/*
impl Operator {
    fn to_string(&self) -> String {
        match self {
            Operator::Equal => "Equal".to_string(),
            Operator::NotEqual => "Not Equal".to_string(),
            Operator::Greater => "Greater".to_string(),
            Operator::Smaller => "Smaller".to_string(),
            Operator::And => "And".to_string(),
            Operator::Or => "Or".to_string(),
        }
    }
}
*/



