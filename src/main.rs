#[allow(unstable_name_collisions)]
mod database;
mod sql;
use crate::sql::*;
use crate::engine::Engine;
use crate::database::database::DB;


fn main() {
    println!("Hello, world!");
    let mut db = DB::new("db.txt").unwrap();
    let sql_s = "SELECT name FROM users WHERE num != 1 AND num != 3";
    let mut lx = sql::lexer::Lexer {
        input: sql_s.chars().peekable(),
        prev_token: SqlToken::EOF,
    };
    let tokens = match lx.lex() {
        Ok(tkns) => tkns,
        Err(e) => panic!("{}", e),
    };
    let mut parser = sql::parser::Parser::new(tokens);
    let ast_root = parser.generate_ast();
    let ast_root = ast_root.unwrap();
    let e = Engine{ast_root: ast_root};
    let _output = e.run(&mut db).unwrap();
    dbg!(_output);
}
