use std::str::Chars;
use crate::sql::*;
use std::iter::Peekable;

#[derive(Debug)]
pub struct Lexer<'a> {
    pub input: Peekable<Chars<'a>>,
    pub prev_token: SqlToken,
}

impl<'a> Lexer<'a> {
    pub fn lex(&mut self) -> Result<Vec<SqlToken>, String> {
        let mut tokens = Vec::new();
        while let Some(c) = self.input.next() {
            match c {
                ' ' | '\n' | '\t' => continue,
                '(' => tokens.push(SqlToken::LeftParen),
                ')' => tokens.push(SqlToken::RightParen),
                ',' => tokens.push(SqlToken::Comma),
                ';' => tokens.push(SqlToken::Semicolon),
                '=' => {
                    if self.prev_token == SqlToken::Operator(Operator::NotEqual) {continue;}
                    tokens.push(SqlToken::Operator(Operator::Equal));
                },
                '!' => {
                    match self.input.peek() {
                        Some(cn) => if *cn == '=' {tokens.push(SqlToken::Operator(Operator::NotEqual))}
                        None => return Err("Unexpected '!' at the end of input".to_string()),
                    }
                },
                '<' => tokens.push(SqlToken::Operator(Operator::Smaller)),
                '>' => tokens.push(SqlToken::Operator(Operator::Greater)),
                '\'' => {
                    let mut s = String::new();
                    while let Some(c) = self.input.next() {
                        if c == '\'' { break; }
                        s.push(c);
                    }
                    tokens.push(SqlToken::StringLiteral(s));
                }

                c if c.is_alphanumeric() => {
                    let mut word = String::new();
                    word.push(c);
                    
                    let mut all_num = c.is_ascii_digit();
                    while let Some(next) = self.input.peek() {
                        if !next.is_alphanumeric() {
                            break;
                        }
                        if next.is_ascii_digit() == false && all_num == true{
                            all_num = false;
                        }
                        word.push(*next);
                        self.input.next();
                    }

                    let token = match word.to_uppercase().as_str() {
                        "SELECT" => SqlToken::Keyword(SqlKeyword::Select),
                        "FROM" => SqlToken::Keyword(SqlKeyword::From),
                        "INSERT" => SqlToken::Keyword(SqlKeyword::Insert),
                        "INTO" => SqlToken::Keyword(SqlKeyword::Into),
                        "VALUES" => SqlToken::Keyword(SqlKeyword::Values),
                        "WHERE" => SqlToken::Keyword(SqlKeyword::Where),
                        "AND" => SqlToken::Keyword(SqlKeyword::And),
                        l => {if all_num {SqlToken::NumberLiteral(l.to_string())} else {SqlToken::Identifier(word)}},
                    };

                    tokens.push(token);
                }
                _ => return Err(format!("Unexpected char: {}", c)),
            }
            self.prev_token = match tokens.last() {
                Some(tkn) => tkn.clone(),
                _ => SqlToken::EOF,
            }
        }

        tokens.push(SqlToken::EOF);
        Ok(tokens)
    }
}

