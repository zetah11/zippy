use num_bigint::BigUint;
use num_traits::FromPrimitive;

use super::{lex, Token};
use crate::source::SourceId;

fn test_lexer(src: &str, expected: &[Token]) {
    let res: Vec<_> = lex(src, SourceId::new())
        .into_iter()
        .map(|(tok, _)| tok)
        .collect();

    assert_eq!(expected, res);
}

#[test]
fn lex_program() {
    let src = r##"
        --- very cool
        -- ok
        --- still cool yeah?
        ---
        ---åops
        main := fun () do
            let x := 5 + 5
        end
    "##;

    let expected = &[
        Token::DocComment("very cool\nstill cool yeah?\n\nåops".into()),
        Token::Name("main".into()),
        Token::Assign,
        Token::Fun,
        Token::LeftParen,
        Token::RightParen,
        Token::Do,
        Token::Let,
        Token::Name("x".into()),
        Token::Assign,
        Token::Integer(BigUint::from_u32(5).unwrap()),
        Token::Plus,
        Token::Integer(BigUint::from_u32(5).unwrap()),
        Token::End,
    ];

    test_lexer(src, expected);
}
