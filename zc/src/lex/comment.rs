//! Parse doc comments, removing the initial `--- `.

use logos::Lexer;

use super::Token;

#[derive(Clone, Copy)]
enum State {
    Initial,
    Normal,
}

/// Remove the triple dashes, the initial space, and the final newline from the doc comment.
pub fn parse_doc_comment(lex: &mut Lexer<Token>) -> String {
    let slice = lex.slice();
    let mut res = String::with_capacity(slice.len() - 4);
    let mut state = State::Initial;

    for c in (&slice[3..]).chars() {
        match (state, c) {
            (State::Initial, ' ') => {
                state = State::Normal;
            }

            (_, '\n') | (_, '\r') => {
                break;
            }

            (State::Initial, c) => {
                state = State::Normal;
                res.push(c);
            }

            (State::Normal, c) => {
                res.push(c);
            }
        }
    }

    res.shrink_to_fit();
    res
}
