#[allow(unstable_name_collisions)]
use flat_file_db::*;

fn main() {
    let mut db = DB::open("/home/macia/Desktop/programming/flat-file-db/test_db").unwrap();
    let sql_s = "SELECT name FROM not_test_table WHERE id > 2";
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
    dbg!(&ast_root);
    let ast_root = ast_root.unwrap();
    let e = Engine{ast_root: ast_root};
    let output = e.run(db).unwrap();
    dbg!(output);
}
