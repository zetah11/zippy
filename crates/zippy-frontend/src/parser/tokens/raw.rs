use logos::{Lexer, Logos};

/// Raw tokens are the "raw" output of the lexer, with comments and indents as
/// separate tokens. This is a very stateless representation, so a second pass
/// is responsible for finding indents/dedents and matching them up correctly,
/// dealing with newlines, properly handling comments, and so on.
#[derive(Logos, Debug)]
pub enum RawToken {
    #[regex(r"\p{XID_Start}[\p{XID_Continue}_']*", |lexer| lexer.slice().to_string())]
    Name(String),

    // Hm.
    #[regex(r"[0-9][0-9_']*(\.[0-9][0-9_']*)?([eE][+\-]?[0-9][0-9_']*)?", |lexer| lexer.slice().to_string())]
    #[regex(r"0b[01][01_']*(\.[01][01_']*)?([eE][+\-]?[01][01_']*)?", |lexer| lexer.slice().to_string())]
    #[regex(r"0x[0-9a-fA-F]", lex_hex_number)]
    Number(String),

    // Stop at the newline for some sensible error recovery
    #[regex(r#""([^"\n\\]|\\.)*["\n]"#, |lexer| lexer.slice().to_string())]
    String(String),

    #[token("fun")]
    Fun,
    #[token("import")]
    Import,
    #[token("let")]
    Let,

    #[token("(")]
    LeftParen,
    #[token(")")]
    RightParen,

    #[token(".")]
    Period,
    #[token(";")]
    Semicolon,
    #[token(":")]
    Colon,

    #[token("=")]
    Equal,

    #[regex(r"--[^\n\r]*", |lexer| lexer.slice().to_string())]
    #[regex(r"-\+", |lexer| lex_block_comment(Block::Normal, lexer))]
    Comment(String),

    #[regex(r"---[^\n\r]*", |lexer| lexer.slice().to_string())]
    #[regex(r"-\+\+", |lexer| lex_block_comment(Block::Doc, lexer))]
    DocComment(String),

    #[regex(r"(\n[ \t]*)+", count_indent)]
    Newline(usize),

    #[error]
    #[regex(r"[ \r\t\v\f]+", logos::skip)]
    Error,
}

#[derive(Clone, Copy)]
enum Block {
    Doc,
    Normal,
}

/// Lex a block comment delimited by `-+ ... +-` or `-++ ... ++-`.
fn lex_block_comment(kind: Block, lexer: &mut Lexer<RawToken>) -> String {
    let mut n = 0;
    let mut plusses = 0;

    for c in lexer.remainder().chars() {
        n += 1;
        match (kind, plusses, c) {
            // End
            (Block::Doc, 2, '-') => break,
            (Block::Normal, 1, '-') => break,

            // Plus
            (Block::Doc, 0 | 1, '+') => plusses += 1,
            (Block::Normal, 0, '+') => plusses += 1,

            // Anything else
            _ => plusses = 0,
        }
    }

    lexer.bump(n);
    lexer.slice().to_string()
}

/// Lex a hexadecimal numeric literal.
fn lex_hex_number(lexer: &mut Lexer<RawToken>) -> String {
    // A hex number consists of (up to) three parts:
    //
    // - The integer part `[0-9a-fA-F][0-9a-fA-F_']*`
    // - The decimal part `\.[0-9a-fA-F][0-9a-fA-F_']*`
    // - The exponent part `[eE][+\-][0-9a-fA-F][0-9a-fA-F_']*`
    //
    // The integer part is mandatory, but the other two are optional. The parts
    // must appear in that order. A regex is sadly not sufficient to parse this,
    // because the exponent part begins with an `e` or `E`, which is also a
    // valid hex digit. This hand crafted state machine thus handles this.

    #[derive(Clone, Copy)]
    enum Part {
        Integer,
        Decimal,
    }

    #[derive(Clone, Copy)]
    enum State {
        Part(Part),
        Dot,
        E(Part),
        PlusMinus,
        ExponentPart,
    }

    // Assume `0x[0-9a-fA-F]` has been consumed by this point.
    let mut n = 0;
    let mut state = State::Part(Part::Integer);

    for c in lexer.remainder().chars() {
        match (state, c) {
            // Exponent shenanigans
            // An `e` or `E` followed by a plus or minus indicates the start of
            // the exponent part.
            (State::E(_), '+' | '-') => {
                state = State::PlusMinus;
            }

            // An `e` or `E` followed by a valid hex digit is also just a valid
            // hex digit.
            (State::E(previous), '0'..='9' | 'a'..='f' | 'A'..='F' | '_' | '\'') => {
                n += 2;
                state = State::Part(previous);
            }

            // An `e` or `E` followed by something else is the end of a hex
            // number.
            (State::E(_), _) => {
                n += 1;
                break;
            }

            // Here, we've matched `[eE][+\-][0-9a-fA-F]`, which means we've
            // successfully moved past three characters.
            (State::PlusMinus, '0'..='9' | 'a'..='f' | 'A'..='F') => {
                n += 3;
                state = State::ExponentPart;
            }

            // Since we had to match an `e` or `E` to get there, it is part of
            // the hex digit (but this `+` or `-` is not), so include it into
            // the token.
            (State::PlusMinus, _) => {
                n += 1;
                break;
            }

            // An `e` or `E` is potentially the start of the exponent part.
            (State::Part(part), 'e' | 'E') => state = State::E(part),

            // Normal digit or separator.
            (
                State::Part(_) | State::ExponentPart,
                '0'..='9' | 'a'..='f' | 'A'..='F' | '_' | '\'',
            ) => {
                n += 1;
            }

            // A dot indicates the start of the decimal part
            (State::Part(Part::Integer), '.') => state = State::Dot,

            // Here, we've matched the two characters `\.[0-9a-fA-F]`
            (State::Dot, '0'..='9' | 'a'..='f' | 'A'..='F') => {
                n += 2;
                state = State::Part(Part::Decimal);
            }

            // Anything else indicates the end of the hex number.
            (State::Part(_) | State::Dot | State::ExponentPart, _) => break,
        }
    }

    lexer.bump(n);
    lexer.slice().to_string()
}

fn count_indent(lexer: &mut Lexer<RawToken>) -> usize {
    let mut spaces = 0;

    for c in lexer.slice().chars() {
        match c {
            '\n' => spaces = 0,
            ' ' | '\t' => spaces += 1,
            _ => unreachable!(),
        }
    }

    spaces
}
