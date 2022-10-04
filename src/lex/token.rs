use logos::Logos;

#[derive(Logos, Debug)]
pub enum FreeToken<'src> {
    #[token("upto")]
    Upto,

    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    #[token("->")]
    MinArrow,

    #[token("=>")]
    EqArrow,

    #[token(":")]
    Colon,

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_']*")]
    Name(&'src str),

    #[regex(r"[0-9][0-9_']*")]
    DecNumber(&'src str),

    #[regex(r"[\n\r][ \t]*", |lex| lex.slice().len() - 1)]
    Newline(usize),

    #[error]
    #[regex(r"[ \t\v\f]+", logos::skip)]
    #[regex(r"--[^\n\r]*", logos::skip)]
    Error,
}
