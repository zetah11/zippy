mod expr;
mod item;

use zippy_common::messages::MessageMaker;
use zippy_common::source::{Source, Span};

use super::tokens::{Token, TokenType};
use crate::Db;

pub struct Parser<'db, I> {
    db: &'db dyn Db,
    tokens: I,

    current: Option<Token>,
    previous: Option<Span>,
    source: Source,
}

impl<'db, I: Iterator<Item = Token>> Parser<'db, I> {
    pub fn new(db: &'db dyn Db, source: Source, tokens: I) -> Self {
        let mut this = Self {
            db,
            tokens,
            current: None,
            previous: None,
            source,
        };

        this.advance();
        this
    }

    /// Get some span close to the current token.
    fn closest_span(&self) -> Span {
        self.current
            .as_ref()
            .map(|token| token.span)
            .or(self.previous)
            .unwrap_or_else(|| self.source.span(0, 0))
    }

    /// Move the parser one token forward.
    fn advance(&mut self) {
        self.previous = self.current.take().map(|token| token.span);
        self.current = self.tokens.next();
    }

    /// Return the span of the current token if it matches.
    fn peek(&self, matcher: impl Matcher) -> Option<Span> {
        let Some(ref current) = self.current else {
            return None;
        };

        matcher.matches(current).then_some(current.span)
    }

    /// Advance the parser and return its span if the current token matches.
    fn consume(&mut self, matcher: impl Matcher) -> Option<Span> {
        let Some(span) = self.peek(matcher) else {
            return None;
        };

        self.advance();
        Some(span)
    }

    /// Skip tokens until the either `current` matches or there are no more
    /// tokens left. Does not skip past the matched token. Returns `true` if
    /// any tokens were skipped.
    fn skip_until(&mut self, matcher: impl Matcher) -> bool {
        let mut skipped = false;

        while let Some(ref token) = self.current {
            if matcher.matches(token) {
                break;
            }

            skipped = true;
            self.advance();
        }

        skipped
    }

    /// Produce a message maker at the given span.
    fn at(&self, span: Span) -> MessageMaker<&'db dyn Db> {
        MessageMaker::new(self.db, span)
    }
}

trait Matcher {
    fn matches(&self, token: &Token) -> bool;
}

impl Matcher for TokenType {
    fn matches(&self, token: &Token) -> bool {
        match (self, &token.kind) {
            (TokenType::Name(_), TokenType::Name(_)) => true,
            (TokenType::Number(_), TokenType::Number(_)) => true,
            (TokenType::String(_), TokenType::String(_)) => true,
            (TokenType::Documentation(_), TokenType::Documentation(_)) => true,

            (a, b) => a == b,
        }
    }
}

impl Matcher for &[TokenType] {
    fn matches(&self, token: &Token) -> bool {
        self.iter().any(|tok| tok.matches(token))
    }
}

impl<const N: usize> Matcher for &[TokenType; N] {
    fn matches(&self, token: &Token) -> bool {
        self.iter().any(|tok| tok.matches(token))
    }
}
