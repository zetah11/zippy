use logos::Logos;
use num_bigint::BigUint;

use super::comment::parse_doc_comment;
use super::number::{parse_int_bin, parse_int_dec, parse_int_hex};
use super::string::parse_string;

/// A token is a small, semantically meaningful chunk of characters that represents some part of
/// the source code.
#[derive(Logos, Debug, Eq, PartialEq)]
pub enum Token {
    /// `break`
    #[token("break")]
    Break,

    /// `continue`
    #[token("continue")]
    Continue,

    /// `return`
    #[token("return")]
    Return,

    /// `do`
    #[token("do")]
    Do,

    /// `end`
    #[token("end")]
    End,

    /// `if`
    #[token("if")]
    If,

    /// `else`
    #[token("else")]
    Else,

    /// `for`
    #[token("for")]
    For,

    /// `in`
    #[token("in")]
    In,

    /// `loop`
    #[token("loop")]
    Loop,

    /// `repeat`
    #[token("repeat")]
    Repeat,

    /// `use`
    #[token("use")]
    Use,

    /// `when`
    #[token("when")]
    When,

    /// `with`
    #[token("with")]
    With,

    /// `as`
    #[token("as")]
    As,

    /// `module`
    #[token("module")]
    Modulue,

    /// `package`
    #[token("package")]
    Package,

    /// `alias`
    #[token("alias")]
    Alias,

    /// `does`
    #[token("does")]
    Does,

    /// `returns`
    #[token("returns")]
    Returns,

    /// `eff`
    #[token("eff")]
    Eff,

    /// `fun`
    #[token("fun")]
    Fun,

    /// `let`
    #[token("let")]
    Let,

    /// `set`
    #[token("set")]
    Set,

    /// `type`
    #[token("type")]
    Type,

    /// `val`
    #[token("val")]
    Val,

    /// `var`
    #[token("var")]
    Var,

    /// `where`
    #[token("where")]
    Where,

    /// `all`
    #[token("all")]
    All,

    /// `any`
    #[token("any")]
    Any,

    /// `bits`
    #[token("bits")]
    Bits,

    /// `digits`
    #[token("digits")]
    Digits,

    /// `mod`
    #[token("mod")]
    Mod,

    /// `thru`
    #[token("thru")]
    Thru,

    /// `upto`
    #[token("upto")]
    Upto,

    /// `and`
    #[token("and")]
    And,

    /// `not`
    #[token("not")]
    Not,

    /// `or`
    #[token("or")]
    Or,

    /// `xor`
    #[token("xor")]
    Xor,

    /// `.`
    #[token(".")]
    Dot,

    /// `..`
    #[token("..")]
    DotDot,

    /// `,`
    #[token(",")]
    Comma,

    /// `:`
    #[token(":")]
    Colon,

    /// `;`
    #[token(";")]
    Semicolon,

    /// `|`
    #[token("|")]
    Pipe,

    /// `&`
    #[token("&")]
    Ampersand,

    /// `?`
    #[token("?")]
    Question,

    /// `???`
    #[token("???")]
    TripleQuestion,

    /// `!`
    #[token("!")]
    Bang,

    /// `#`
    #[token("#")]
    Hash,

    /// `@`
    #[token("@")]
    At,

    /// `+`
    #[token("+")]
    Plus,

    /// `-`
    #[token("-")]
    Minus,

    /// `*`
    #[token("*")]
    Star,

    /// `/`
    #[token("/")]
    Slash,

    /// `**`
    #[token("**")]
    DoubleStar,

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

    /// Assignment `:=`
    #[token(":=")]
    Assign,

    /// Type declaration `::`
    #[token("::")]
    Declare,

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

    /// Some regex literal like `r/.*/`
    #[regex(r"r/([^\n\r/\\]|\\[^\n\r])/", parse_string)]
    Regex(String),

    /// Some string literal like `"abc\"ok\"!\n"` or `'hello!'`
    #[regex(r#""([^\n\r"\\]|\\[^\n\r])*""#, parse_string)]
    #[regex(r"'([^\n\r'\\]|\\[^\n\r])*'", parse_string)]
    Text(String),

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

    /// One or more lineshifts. May be merged with a doccomment.
    #[regex(r"[\r\n]+")]
    #[regex(r"--[^\n\r]*(\r\n|\n|\r)?")]
    Newline,

    /// Some invalid or unexpected token.
    #[error]
    #[regex(r"[ \t\f]+", logos::skip)]
    #[regex(r"-\+.*\+-", logos::skip)]
    Error,
}
