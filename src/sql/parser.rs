use crate::sql::*;
use crate::sql::ast::*;

#[derive(Debug)]
pub struct Parser {
    tokens: Vec<SqlToken>,
    pos: usize,
}

impl Parser {
    pub fn new(vec: Vec<SqlToken>) -> Self {
        Parser {tokens: vec, pos: 0}
    }
    fn current(&self) -> &SqlToken {
        &self.tokens[self.pos]
    }

    fn advance(&mut self) {
        self.pos += 1;
    }
    fn parse_comparison(&mut self) -> Result<Expr, String> {
        let left = self.parse_primary()?;

        if let SqlToken::Operator(op @ (
                Operator::Equal |
                Operator::NotEqual |
                Operator::Greater |
                Operator::Smaller
        )) = self.current() {
            let op = op.clone();
            self.advance();
            let right = self.parse_primary()?;

            Ok(Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            })
        } else {
            Ok(left)
        }
    }

    fn parse_expr(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_comparison()?;

        while let SqlToken::Operator(op @ (Operator::And | Operator::Or)) = self.current() {
            let op = op.clone();
            self.advance();
            let right = self.parse_comparison()?;

            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }


    fn expect(&mut self, expected: SqlToken) -> Result<(), String> {
        if *self.current() == expected {
            self.advance();
            Ok(())
        } else {
            Err(format!("Expected {:?}, got {:?}", expected, self.current()))
        }
    }
    fn parse_comma_separated<T>(&mut self, mut parse_item: impl FnMut(&mut Self) -> Result<T, String> ) -> Result<Vec<T>, String> {
        let mut items = Vec::new();
        items.push(parse_item(self)?);

        while self.current() == &SqlToken::Comma {
            self.advance();
            items.push(parse_item(self)?);
        }

        Ok(items)
    }
    pub fn parse_statement(&mut self) -> Result<Statement, String> {
        match self.current() {
            SqlToken::Keyword(SqlKeyword::Select) => {
                Ok(Statement::Select(self.parse_select()?))
            }
            SqlToken::Keyword(SqlKeyword::Insert) => {
                Ok(Statement::Insert(self.parse_insert()?))
            }
            token => Err(format!("Unexpected token {:?}", token)),
        }
    }
    fn parse_identifier(&mut self) -> Result<String, String> {
        match self.current() {
            SqlToken::Identifier(name) => {
                let name = name.clone();
                self.advance();
                Ok(name)
            }
            token => Err(format!("Expected identifier, found {:?}", token)),
        }
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
       match self.current() {
            SqlToken::StringLiteral(s) => {
                let expr = Expr::Literal(ast::Literal::String(s.clone()));
                self.advance();
                Ok(expr)
            }
            SqlToken::NumberLiteral(n) => {
                let value = n.parse::<i32>()
                    .map_err(|_| format!("Invalid number literal: {}", n))?;
                self.advance();
                Ok(Expr::Literal(ast::Literal::Number(value)))
            }
            SqlToken::Identifier(name) => {
                let expr = Expr::Identifier(name.clone());
                self.advance();
                Ok(expr)
            }
            token => Err(format!("Expected expression, found {:?}", token)),
        }
    }

    fn parse_select(&mut self) -> Result<SelectStatement, String> {
        self.expect(SqlToken::Keyword(SqlKeyword::Select))?;

        let columns = self.parse_comma_separated(|p| p.parse_identifier())?;
        self.expect(SqlToken::Keyword(SqlKeyword::From))?;

        let table = self.parse_identifier()?;
        let mut where_clauses: Vec<Expr> = vec![];
        loop {
            let where_clause = if self.current() == &SqlToken::Keyword(SqlKeyword::Where) ||
                self.current() == &SqlToken::Operator(Operator::And) {
                    self.advance(); 
                    match self.parse_expr() {
                        Ok(wc) => wc,
                        Err(_s) => break,
                    }
                } else {
                    break;
            };
            where_clauses.push(where_clause);
        }
        let where_clauses = if where_clauses.len() < 1 {
            Option::None
        } else {
            Option::Some(where_clauses)
        };

        Ok(SelectStatement {
            columns,
            table,
            where_clause: where_clauses,
        })
    }
    pub fn parse_insert(&mut self) -> Result<InsertStatement, String> {
        self.expect(SqlToken::Keyword(SqlKeyword::Insert))?;
        self.expect(SqlToken::Keyword(SqlKeyword::Into))?;

        let table = match self.current() {
            SqlToken::Identifier(name) => {
                let t = name.clone();
                self.advance();
                t
            }
            _ => return Err("Expected table name".into()),
        };
        self.expect(SqlToken::LeftParen)?;
        let columns = self.parse_comma_separated(|p| p.parse_identifier())?;
        self.expect(SqlToken::RightParen)?;

        self.expect(SqlToken::Keyword(SqlKeyword::Values))?;
        self.expect(SqlToken::LeftParen)?;

        let mut values = Vec::new();
        while *self.current() != SqlToken::RightParen {
            if *self.current() == SqlToken::RightParen {break}
            values.push(self.parse_expr()?);

            if *self.current() == SqlToken::Comma {
                self.advance();
            }
        }
        self.expect(SqlToken::RightParen)?;
        Ok(InsertStatement {columns: Some(columns), table, values})
    }
    pub fn generate_ast(&mut self) -> Result<ASTRootWrapper, String> {
        let base_node = match self.tokens.get(0) {
            Some(s) => match s {
                SqlToken::Keyword(SqlKeyword::Select) => {ASTRootWrapper{first_node: ASTNode::Statment(Statement::Select(self.parse_select()?))}},
                SqlToken::Keyword(SqlKeyword::Insert) => {ASTRootWrapper{first_node: ASTNode::Statment(Statement::Insert(self.parse_insert()?))}},
                _ => return Err("error, expected keyword at the first position".to_string()),
            },
            _ => return Err("error, expected input".to_string()),

        };
        Ok(base_node)
    }
}
