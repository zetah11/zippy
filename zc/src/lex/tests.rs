use num_bigint::BigUint;
use num_traits::FromPrimitive;

use super::{lex_src, Token};
use crate::source::SourceId;

fn test_lexer(src: &str, expected: &[Token]) {
    let res: Vec<_> = lex_src(src, SourceId::new())
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
        fun main()
            let x = 5 + 5
        end
    "##;

    let expected = &[
        Token::DocComment("very cool\nstill cool yeah?\n\nåops".into()),
        Token::Fun,
        Token::Name("main".into()),
        Token::LeftParen,
        Token::RightParen,
        Token::Let,
        Token::Name("x".into()),
        Token::Equal,
        Token::Integer(BigUint::from_u32(5).unwrap()),
        Token::Plus,
        Token::Integer(BigUint::from_u32(5).unwrap()),
        Token::End,
    ];

    test_lexer(src, expected);
}

#[test]
fn lex_literals() {
    let src = r##"
        r/.* ok\// 5 "hello!" 'hello!'
    "##;

    let expected = &[
        Token::Regex(".* ok/".into()),
        Token::Integer(BigUint::from_u32(5).unwrap()),
        Token::String("hello!".into()),
        Token::String("hello!".into()),
    ];

    test_lexer(src, expected);
}
