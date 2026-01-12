pub mod lexer;
pub mod parser;
pub mod ast;
pub mod engine;

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
    And,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    Equal,
    NotEqual,
    Greater,
    Smaller,
    Plus,
}

impl Operator {
    fn to_string(&self) -> String {
        match self {
            Operator::Equal => "Equal".to_string(),
            Operator::NotEqual => "Not Equal".to_string(),
            Operator::Greater => "Greater".to_string(),
            Operator::Smaller => "Smaller".to_string(),
            Operator::Plus => "Plus".to_string(),
        }
    }

}



