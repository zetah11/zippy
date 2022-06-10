//! Lexing is the process of turning a string of characters into a stream of tokens. A token is
//! some chunk of characters that has some meaning by itself, such as a string literal, an
//! operator, a name, or a keyword.

mod comment;
mod number;
mod string;
mod token;

#[cfg(test)]
mod tests;

pub use lexer_def::{Lexer, LexerStorage};
pub use token::Token;

mod lexer_def {
    use super::Token;

    use std::sync::Arc;

    use logos::Logos;

    use crate::inputs::Inputs;
    use crate::source::{SourceId, Span};

    /// See the [module-level documentation](crate::lex) for more.
    #[salsa::query_group(LexerStorage)]
    pub trait Lexer: Inputs {
        /// Tokenize the given source.
        fn lex(&self, id: SourceId) -> Arc<Vec<(Token, Span)>>;
    }

    fn lex(db: &dyn Lexer, id: SourceId) -> Arc<Vec<(Token, Span)>> {
        let src = db.input_file(id);
        Arc::new(lex_src(src.as_ref(), id))
    }

    /// Lex and tokenize the given source text.
    pub(super) fn lex_src(src: impl AsRef<str>, source: SourceId) -> Vec<(Token, Span)> {
        let mut res = Vec::new();
        let mut prev: Option<(Token, Span)> = None;

        for (tok, span) in Token::lexer(src.as_ref()).spanned() {
            let span = source.span(span.start, span.end);

            match (prev, tok) {
                (Some(tok), Token::Error) => {
                    prev = Some(tok);
                }

                (Some((Token::DocComment(mut doc1), span1)), Token::DocComment(doc2)) => {
                    doc1.push('\n');
                    doc1.push_str(&doc2);
                    prev = Some((Token::DocComment(doc1), span1.combine(&span)));
                }

                // Push the previous token
                (Some(to_store), curr) => {
                    res.push(to_store);
                    prev = Some((curr, span));
                }

                // First token
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
}
