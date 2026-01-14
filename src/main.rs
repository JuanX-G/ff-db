#[allow(unstable_name_collisions)]
mod database;
mod sql;
use std::env::current_dir;
use crate::sql::*;
use crate::engine::Engine;
use crate::database::database::DB;


fn main() {
    println!("Hello, world!");
    let mut db = DB::open("./db").unwrap();
    let tab = match db.get_mut_table("db.txt") {
        Some(tb) => tb,
        None => panic!("did not find table"),
    };
    let sql_s = "SELECT name FROM users WHERE num != 1 AND num != 2";
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
    let output = e.run(tab).unwrap();
    dbg!(output);
}
