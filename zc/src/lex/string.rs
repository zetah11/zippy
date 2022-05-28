//! Methods for parsing a raw string text like `"ab\"\n"` into literal string value (mainly just
//! parsing escapes).

use logos::Lexer;

use super::Token;

#[derive(Clone, Copy)]
enum State {
    Escaped,
    Normal,
}

/// Parse the contents (escapes) of a string.
pub fn parse_string(lex: &mut Lexer<Token>) -> String {
    let slice = lex.slice();
    let slice = &slice[1..slice.len() - 1];

    // Let's limit ourselves to one reallocation (in the case of escapes - otherwise, no
    // reallocation is necessary)
    let mut res = String::with_capacity(slice.len());
    let mut state = State::Normal;

    for c in slice.chars() {
        match (state, c) {
            (State::Escaped, 'n') => {
                res.push('\n');
                state = State::Normal
            }
            (State::Escaped, 'r') => {
                res.push('\r');
                state = State::Normal
            }
            (State::Escaped, 't') => {
                res.push('\t');
                state = State::Normal
            }
            (State::Escaped, c) => {
                res.push(c);
                state = State::Normal
            }

            (State::Normal, '\\') => state = State::Escaped,
            (State::Normal, c) => res.push(c),
        }
    }

    res.shrink_to_fit();
    res
}
