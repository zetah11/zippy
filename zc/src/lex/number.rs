//! Parse numeric (integer and decimal) literals.

use logos::Lexer;
use num_bigint::BigUint;
use num_traits::Num;

use super::Token;

pub fn parse_int_dec(lex: &mut Lexer<Token>) -> BigUint {
    BigUint::from_str_radix(lex.slice(), 10).unwrap()
}

pub fn parse_int_bin(lex: &mut Lexer<Token>) -> BigUint {
    BigUint::from_str_radix(lex.slice(), 2).unwrap()
}

pub fn parse_int_hex(lex: &mut Lexer<Token>) -> BigUint {
    BigUint::from_str_radix(lex.slice(), 16).unwrap()
}
