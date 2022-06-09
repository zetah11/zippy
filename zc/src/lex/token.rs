use std::fmt;

use logos::Logos;
use num_bigint::BigUint;

use super::comment::parse_doc_comment;
use super::number::{parse_int_bin, parse_int_dec, parse_int_hex};
use super::string::{parse_regex, parse_string};

/// A token is a small, semantically meaningful chunk of characters that represents some part of
/// the source code.
#[derive(Logos, Clone, Debug, Eq, PartialEq)]
pub enum Token {
    // Keywords
    /// `and`
    #[token("and")]
    And,

    /// `do`
    #[token("do")]
    Do,

    /// `else`
    #[token("else")]
    Else,

    /// `end`
    #[token("end")]
    End,

    /// `false`
    #[token("false")]
    False,

    /// `fun`
    #[token("fun")]
    Fun,

    /// `if`
    #[token("if")]
    If,

    /// `let`
    #[token("let")]
    Let,

    /// `mod`
    #[token("mod")]
    Mod,

    /// `module`
    #[token("module")]
    Module,

    /// `not`
    #[token("not")]
    Not,

    /// `or`
    #[token("or")]
    Or,

    /// `return`
    #[token("return")]
    Return,

    /// `thru`
    #[token("then")]
    Then,

    /// `thru`
    #[token("thru")]
    Thru,

    /// `true`
    #[token("true")]
    True,

    /// `type`
    #[token("type")]
    Type,

    /// `upto`
    #[token("upto")]
    Upto,

    /// `var`
    #[token("var")]
    Var,

    /// `where`
    #[token("where")]
    Where,

    /// `xor`
    #[token("xor")]
    Xor,

    // Punctuation
    /// `.`
    #[token(".")]
    Dot,

    /// `,`
    #[token(",")]
    Comma,

    /// `;`
    #[token(";")]
    Semicolon,

    // Operators
    /// `:`
    #[token(":")]
    Colon,

    /// `?`
    #[token("?")]
    Question,

    /// `+`
    #[token("+")]
    Plus,

    /// `-`
    #[token("-")]
    Minus,

    /// `*`
    #[token("*")]
    Star,

    /// `**`
    #[token("**")]
    DoubleStar,

    /// `/`
    #[token("/")]
    Slash,

    /// `=`
    #[token("=")]
    Equal,

    /// `/=`
    #[token("/=")]
    SlashEqual,

    /// `<`
    #[token("<")]
    Less,

    /// `<=`
    #[token("<=")]
    LessEqual,

    /// `>`
    #[token(">")]
    Greater,

    /// `>=`
    #[token(">=")]
    GreaterEqual,

    // Grouping
    /// `(`
    #[token("(")]
    LeftParen,

    /// `)`
    #[token(")")]
    RightParen,

    /// `[`
    #[token("[")]
    LeftBracket,

    /// `]`
    #[token("]")]
    RightBracket,

    /// `{`
    #[token("{")]
    LeftBrace,

    /// `}`
    #[token("}")]
    RightBrace,

    // Literals
    /// Some regex literal like `r/.*/`
    #[regex(r"r/([^\n\r/\\]|\\[^\n\r])*/", parse_regex)]
    Regex(String),

    /// Some string literal like `"abc\"ok\"!\n"` or `'hello!'`
    #[regex(r#""([^\n\r"\\]|\\[^\n\r])*""#, parse_string)]
    #[regex(r"'([^\n\r'\\]|\\[^\n\r])*'", parse_string)]
    String(String),

    /// Some integer literal, like `0x123` or `0b11_011`
    #[regex(r"[0-9][0-9_]*", parse_int_dec)]
    #[regex(r"0b[01][01_]*", parse_int_bin)]
    #[regex(r"0x[0-9a-fA-F][0-9a-fA-F_]*", parse_int_hex)]
    Integer(BigUint),

    /// Some decimal literal, like `12.34e-56`
    #[regex(r"[0-9][0-9_]*\.[0-9][0-9_]*", |lex| lex.slice().to_string())]
    #[regex(r"[0-9][0-9_]*[eE][+\-]?[0-9][0-9_]*", |lex| lex.slice().to_string())]
    #[regex(r"[0-9][0-9_]*\.[0-9][0-9_]*[eE][+\-]?[0-9][0-9_]*", |lex| lex.slice().to_string())]
    Decimal(String),

    /// Names identify program items.
    #[regex(r"[a-zA-Z][a-zA-Z0-9_?!]*", |lex| lex.slice().to_string())]
    Name(String),

    /// Some documentation for an item.
    #[regex(r"---[^\n\r]*(\r\n|\n|\r)?", parse_doc_comment)]
    DocComment(String),

    /// Some invalid or unexpected token.
    #[error]
    #[regex(r"[ \t\f\r\n]+", logos::skip)]
    #[regex(r"-\+.*\+-", logos::skip)]
    #[regex(r"--[^\n\r]*(\r\n|\n|\r)?", logos::skip)]
    Error,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tok = match self {
            Token::And => "and",
            Token::Do => "do",
            Token::Else => "else",
            Token::End => "end",
            Token::False => "false",
            Token::Fun => "fun",
            Token::If => "if",
            Token::Let => "let",
            Token::Mod => "mod",
            Token::Module => "module",
            Token::Not => "not",
            Token::Or => "or",
            Token::Return => "return",
            Token::Then => "then",
            Token::Thru => "thru",
            Token::True => "true",
            Token::Type => "type",
            Token::Upto => "upto",
            Token::Var => "var",
            Token::Where => "where",
            Token::Xor => "xor",

            Token::Dot => ".",
            Token::Comma => ",",
            Token::Semicolon => ";",
            Token::Colon => ":",
            Token::Question => "?",
            Token::Plus => "+",
            Token::Minus => "-",
            Token::Star => "*",
            Token::DoubleStar => "**",
            Token::Slash => "/",
            Token::Equal => "=",
            Token::SlashEqual => "/=",
            Token::Less => "<",
            Token::LessEqual => "<=",
            Token::Greater => ">",
            Token::GreaterEqual => ">=",

            Token::LeftParen => "(",
            Token::RightParen => ")",
            Token::LeftBracket => "[",
            Token::RightBracket => "]",
            Token::LeftBrace => "{",
            Token::RightBrace => "}",

            // TODO: output the actual lexeme?
            Token::Regex(_) => "<regex>",
            Token::String(_) => "<string>",
            Token::Integer(_) => "<integer>",
            Token::Decimal(_) => "<decimal>",
            Token::Name(_) => "<name>",
            Token::DocComment(_) => "<doc comment>",
            Token::Error => "<error>",
        };
        write!(f, "{}", tok)
    }
}
