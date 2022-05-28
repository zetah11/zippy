//! Lexing is the process of turning a string of characters into a stream of tokens. A token is
//! some chunk of characters that has some meaning by itself, such as a string literal, an
//! operator, a name, or a keyword.

mod comment;
mod number;
mod string;
mod token;

#[cfg(test)]
mod tests;

pub use token::Token;

use logos::Logos;

use crate::source::{SourceId, Span};

/// Lex and tokenize the given source text.
pub fn lex(src: impl AsRef<str>, source: SourceId) -> Vec<(Token, Span)> {
    let mut res = Vec::new();
    let mut prev: Option<(Token, Span)> = None;

    for (tok, span) in Token::lexer(src.as_ref()).spanned() {
        let span = source.span(span.start, span.end);

        match (prev, tok) {
            (Some((Token::DocComment(mut doc1), span1)), Token::DocComment(doc2)) => {
                doc1.push('\n');
                doc1.push_str(&doc2);
                prev = Some((Token::DocComment(doc1), span1.combine(&span)));
            }

            (Some(to_store), curr) => {
                res.push(to_store);
                prev = Some((curr, span));
            }

            (None, curr) => {
                prev = Some((curr, span));
            }
        }
    }

    if let Some(prev) = prev {
        res.push(prev);
    }

    res
}
