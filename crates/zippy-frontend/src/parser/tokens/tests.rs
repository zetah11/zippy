use zippy_common::messages::Messages;
use zippy_common::source::Source;

use super::{get_tokens, TokenType};

#[derive(Default)]
#[salsa::db(crate::Jar, zippy_common::Jar)]
struct MockDb {
    storage: salsa::Storage<Self>,
}

impl salsa::Database for MockDb {}

/// Check that the lexer produces the expected token types, and that no messages
/// are emitted.
fn check(source: impl Into<String>, expected: &[TokenType]) {
    let db = MockDb::default();
    let source = Source::new(&db, "test".into(), source.into());

    let tokens = get_tokens(&db, source);

    assert_eq!(expected.len(), tokens.len());

    for (expected, actual) in expected.iter().zip(tokens) {
        assert_eq!(expected, &actual.kind);
    }

    let messages = get_tokens::accumulated::<Messages>(&db, source);
    assert!(messages.is_empty());
}

#[test]
fn lex_dedent_at_eof() {
    let source = "let x =\n  y";
    let expected = &[
        TokenType::Let,
        TokenType::Name("x".into()),
        TokenType::Equals,
        TokenType::Indent,
        TokenType::Name("y".into()),
        TokenType::Dedent,
    ];

    check(source, expected);
}

#[test]
fn lex_simple_eols() {
    let source = "let x\nlet y\nlet z";
    let expected = &[
        TokenType::Let,
        TokenType::Name("x".into()),
        TokenType::Eol,
        TokenType::Let,
        TokenType::Name("y".into()),
        TokenType::Eol,
        TokenType::Let,
        TokenType::Name("z".into()),
    ];

    check(source, expected);
}

#[test]
fn lex_dedents_and_eols() {
    let source = "let x =\n  let y = 5\n  let z = y\n  z\nlet w";
    let expected = &[
        TokenType::Let,
        TokenType::Name("x".into()),
        TokenType::Equals,
        TokenType::Indent,
        TokenType::Let,
        TokenType::Name("y".into()),
        TokenType::Equals,
        TokenType::Number("5".into()),
        TokenType::Eol,
        TokenType::Let,
        TokenType::Name("z".into()),
        TokenType::Equals,
        TokenType::Name("y".into()),
        TokenType::Eol,
        TokenType::Name("z".into()),
        TokenType::Dedent,
        TokenType::Eol,
        TokenType::Let,
        TokenType::Name("w".into()),
    ];

    check(source, expected);
}

#[test]
fn lex_dedents_not_followed_by_dedents() {
    let source = "\n  x\n y\nz";
    let expected = &[
        TokenType::Indent,
        TokenType::Name("x".into()),
        TokenType::Dedent,
        TokenType::Eol,
        TokenType::Name("y".into()),
        TokenType::Eol,
        TokenType::Name("z".into()),
    ];

    check(source, expected);
}

#[test]
fn lex_keywords() {
    let source = "fun let";
    let expected = &[TokenType::Fun, TokenType::Let];

    check(source, expected);
}

#[test]
fn lex_strings() {
    let source = r#" "abc" "de\"" "heep hoo\n" "#;
    let expected = &[
        TokenType::String(r#""abc""#.into()),
        TokenType::String(r#""de\"""#.into()),
        TokenType::String(r#""heep hoo\n""#.into()),
    ];

    check(source, expected);
}

#[test]
fn lex_decimal_whole_numbers() {
    let source = "0 123 019 01_'2'___''''3456789_'_'";
    let expected = &[
        TokenType::Number("0".into()),
        TokenType::Number("123".into()),
        TokenType::Number("019".into()),
        TokenType::Number("01_'2'___''''3456789_'_'".into()),
    ];

    check(source, expected);
}

#[test]
fn lex_decimal_point_numbers() {
    let source = "0.1 0.1_ 12''.3 1_2_'.3'''''5''9";
    let expected = &[
        TokenType::Number("0.1".into()),
        TokenType::Number("0.1_".into()),
        TokenType::Number("12''.3".into()),
        TokenType::Number("1_2_'.3'''''5''9".into()),
    ];

    check(source, expected);
}

#[test]
fn lex_decimal_exponent_numbers() {
    let source = "1e4 0e+0 1_e-5 0''e+0'''";
    let expected = &[
        TokenType::Number("1e4".into()),
        TokenType::Number("0e+0".into()),
        TokenType::Number("1_e-5".into()),
        TokenType::Number("0''e+0'''".into()),
    ];

    check(source, expected);
}

#[test]
fn lex_decimal_full_number() {
    let source = "1.23e45 0_.0'e-0_' 1_23'.4_56'E+7_89'";
    let expected = &[
        TokenType::Number("1.23e45".into()),
        TokenType::Number("0_.0'e-0_'".into()),
        TokenType::Number("1_23'.4_56'E+7_89'".into()),
    ];

    check(source, expected);
}

#[test]
fn lex_binary_whole_numbers() {
    let source = "0b0 0b0101 0b0_1_0'''1_012";
    let expected = &[
        TokenType::Number("0b0".into()),
        TokenType::Number("0b0101".into()),
        TokenType::Number("0b0_1_0'''1_01".into()),
        TokenType::Number("2".into()),
    ];

    check(source, expected);
}

#[test]
fn lex_binary_point_numbers() {
    let source = "0b0.1 0b0_0.1''";
    let expected = &[
        TokenType::Number("0b0.1".into()),
        TokenType::Number("0b0_0.1''".into()),
    ];

    check(source, expected);
}

#[test]
fn lex_binary_exponent_numbers() {
    let source = "0b0e0101 0b0_1'e-11''0";
    let expected = &[
        TokenType::Number("0b0e0101".into()),
        TokenType::Number("0b0_1'e-11''0".into()),
    ];

    check(source, expected);
}

#[test]
fn lex_binary_full_numbers() {
    let source = "0b101.010e-110 0b1_0'1.0'1_1E+101'";
    let expected = &[
        TokenType::Number("0b101.010e-110".into()),
        TokenType::Number("0b1_0'1.0'1_1E+101'".into()),
    ];

    check(source, expected);
}

#[test]
fn lex_hex_whole_numbers() {
    let source = "0x0123456789aBcDeFg 0x1_b'AAAAA";
    let expected = &[
        TokenType::Number("0x0123456789aBcDeF".into()),
        TokenType::Name("g".into()),
        TokenType::Number("0x1_b'AAAAA".into()),
    ];

    check(source, expected);
}

#[test]
fn lex_hex_point_numbers() {
    let source = "0x123.aFb_''' 0x1_2''.3 0xFf.Ff";
    let expected = &[
        TokenType::Number("0x123.aFb_'''".into()),
        TokenType::Number("0x1_2''.3".into()),
        TokenType::Number("0xFf.Ff".into()),
    ];

    check(source, expected);
}

#[test]
fn lex_hex_exponent_numbers() {
    let source = "0xfe+f 0xFe-F 0xEE+E";
    let expected = &[
        TokenType::Number("0xfe+f".into()),
        TokenType::Number("0xFe-F".into()),
        TokenType::Number("0xEE+E".into()),
    ];

    check(source, expected);
}

#[test]
fn lex_hex_full_numbers() {
    let source = "0x12_345.678'9aE+bcDEf'''";
    let expected = &[TokenType::Number("0x12_345.678'9aE+bcDEf'''".into())];

    check(source, expected);
}
