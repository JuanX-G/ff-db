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



