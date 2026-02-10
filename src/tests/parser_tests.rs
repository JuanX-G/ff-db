use crate::sql;
use crate::SqlToken;
use crate::SqlKeyword;

#[test]
fn test_lexing() {
    let sql_s = "SELECT name FROM test_table";
    let mut lx = sql::lexer::Lexer {
        input: sql_s.chars().peekable(),
        prev_token: SqlToken::EOF,
    };
    let tokens = match lx.lex() {
        Ok(tkns) => tkns,
        Err(e) => panic!("{}", e),
    };
    let expected_vec = vec![SqlToken::Keyword(SqlKeyword::Select), 
        SqlToken::Identifier("name".to_string()),
        SqlToken::Keyword(SqlKeyword::From),
        SqlToken::Identifier("test_table".to_string()),
        SqlToken::EOF
    ];
    assert_eq!(tokens, expected_vec);
}
