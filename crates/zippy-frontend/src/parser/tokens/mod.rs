//! Tokenization is the process of converting a string of characters into a
//! string of tokens (which are like the "words" of the programming language).
//! Because zippy is whitespace sensitive, the tokenization pass is a little bit
//! more complicated than usual. Specifically, it is a two-pass process where
//! the string is first tokenized into [raw tokens](raw), which are then
//! processed to create the appropriate `Indent`, `Dedent` and `Eol` tokens.
//!
//! The first pass produces a token `Newline(usize)` when it encounters a
//! newline, with the `usize` counting the number of spaces it is immediately
//! followed by. The second pass then uses this information (taking only the
//! last one if several appear in a row) to produce indentation information.
//!
//! - If the indentation level **increases**, a single `Indent` token is
//!   produced, and this new level is remember in a stack of indentation levels.
//! - If the indentation level **decreases**, `Dedent` tokens are emitted until
//!   the top of the indentation stack is greater or equal.
//! - If the indentation level is unchanged or has decreased, an `Eol` token
//!   is emitted after every `Dedent` token.
//! - At the end of the source, `Dedents` are emitted to clear the indentation
//!   stack, followed by a single `Eol` if any tokens were produced.
//!
//! Thus, code like
//!
//! ```zippy
//! fun f x =
//!   let y =
//!     x +
//!       2
//!   y
//! ```
//!
//! produces a token stream like
//!
//! ```text
//! Fun, Name, Name, Equal,
//! Indent,
//! Let, Name, Equal,
//! Indent,
//! Name, Plus,
//! Indent,
//! Number,
//! Dedent,
//! Dedent,
//! Eol,
//! Name,
//! Dedent,
//! Eol
//! ```

mod raw;

#[cfg(test)]
mod tests;

use std::collections::VecDeque;

use logos::Logos;
use zippy_common::messages::MessageMaker;
use zippy_common::source::{Source, Span};

use self::raw::RawToken;
use crate::messages::ParseMessages;
use crate::Db;

#[salsa::tracked]
pub fn get_tokens(db: &dyn crate::Db, src: Source) -> Vec<Token> {
    let db = <dyn Db as salsa::DbWithJar<zippy_common::Jar>>::as_jar_db(db);
    TokenIter::new(src, src.content(db))
        .filter_map(|token| match token {
            Ok(token) => Some(token),
            Err(span) => {
                MessageMaker::new(db, span).unexpected_token();
                None
            }
        })
        .collect()
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Token {
    pub kind: TokenType,

    /// Any preceding (non-doc) comments.
    pub comments: Vec<(String, Span)>,

    /// The span of this token.
    pub span: Span,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum TokenType {
    Indent,
    Dedent,
    Eol,

    Name(String),
    Number(String),
    String(String),

    Fun,
    Let,

    Equals,

    LeftParen,
    RightParen,
    Semicolon,

    Documentation(String),
}

/// An iterator producing a stream of tokens from a given source. This produces
/// a result which is either `Ok(token)` with a token or `Err(span)` when an
/// invalid token was encountered at the given span.
struct TokenIter<'source> {
    lexer: logos::SpannedIter<'source, RawToken>,
    source: Source,

    indents: (Vec<usize>, usize),
    last_span: Span,

    last_tokens: VecDeque<Token>,
}

impl<'source> TokenIter<'source> {
    pub fn new(source: Source, content: &'source str) -> Self {
        Self {
            lexer: RawToken::lexer(content).spanned(),
            source,

            indents: (Vec::new(), 0),
            last_span: source.span(0, 0),

            last_tokens: VecDeque::new(),
        }
    }

    /// Get the tokens produced by this newline, if any. This produces tokens in
    /// the following order:
    ///
    /// - Any number of `Dedent`s (while popping from the dedent stack)
    /// - Possibly one `Eol`
    /// - Possibly one `Indent` (while pushing to the indent stack)
    fn newline_tokens(&mut self, indent: (usize, Span)) -> VecDeque<Token> {
        let mut tokens = VecDeque::new();
        let new_indent = indent.0.cmp(&self.indents.1);

        // If new indent is less than current indent, emit dedent tokens until
        // its not.
        while indent.0 < self.indents.1 {
            tokens.push_back(Token {
                kind: TokenType::Dedent,
                comments: Vec::new(),
                span: indent.1,
            });

            if let Some(topmost) = self.indents.0.pop() {
                self.indents.1 = topmost;
            } else {
                assert_eq!(0, self.indents.1);
                break;
            }
        }

        // Only emit a semicolon if new indent is less than or equal to the
        // current indent and we plausibly expect an eol here.
        if new_indent.is_le() {
            tokens.push_back(Token {
                kind: TokenType::Eol,
                comments: Vec::new(),
                span: indent.1,
            });
        }

        // Emit a single indent if indentation level increased.
        if new_indent.is_gt() {
            self.indents.0.push(self.indents.1);
            self.indents.1 = indent.0;
            tokens.push_back(Token {
                kind: TokenType::Indent,
                comments: Vec::new(),
                span: indent.1,
            });
        }

        tokens
    }
}

impl Iterator for TokenIter<'_> {
    type Item = Result<Token, Span>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut last_indent = None;
        let mut comments = Vec::new();

        loop {
            let (token, span) = match self.last_tokens.pop_front() {
                Some(token) => return Some(Ok(token)),
                None => match self.lexer.next() {
                    Some((token, span)) => (token, self.source.span(span.start, span.end)),
                    None => {
                        if let Some(front) = self.indents.0.pop() {
                            let token = Token {
                                kind: TokenType::Dedent,
                                comments: Vec::new(),
                                span: self.last_span,
                            };

                            self.indents.1 = front;

                            return Some(Ok(token));
                        } else {
                            return None;
                        }
                    }
                },
            };

            self.last_span = span;

            let kind = match token {
                RawToken::Newline(indent) => {
                    last_indent = Some((indent, span));
                    continue;
                }

                RawToken::Comment(comment) => {
                    comments.push((comment, span));
                    continue;
                }

                RawToken::DocComment(doc) => TokenType::Documentation(doc),

                RawToken::Name(name) => TokenType::Name(name),
                RawToken::Number(number) => TokenType::Number(number),
                RawToken::String(string) => TokenType::String(string),

                RawToken::Fun => TokenType::Fun,
                RawToken::Let => TokenType::Let,

                RawToken::Equals => TokenType::Equals,

                RawToken::LeftParen => TokenType::LeftParen,
                RawToken::RightParen => TokenType::RightParen,
                RawToken::Semicolon => TokenType::Semicolon,

                RawToken::Error => {
                    return Some(Err(span));
                }
            };

            let token = Token {
                kind,
                comments,
                span,
            };

            let token = if let Some(indent) = last_indent {
                let mut tokens = self.newline_tokens(indent);
                if let Some(front) = tokens.pop_front() {
                    tokens.push_back(token);
                    self.last_tokens.extend(tokens);
                    front
                } else {
                    token
                }
            } else {
                token
            };

            return Some(Ok(token));
        }
    }
}
