pub mod tree;

mod decl;
mod expr;
mod matcher;
mod unconcretify;

use common::hir::Decls;
use common::message::{File, Messages, Span};
use common::Driver;
use log::{info, trace};

use crate::lex::Token;
use matcher::Matcher;
use unconcretify::Unconcretifier;

pub fn parse(
    driver: &mut impl Driver,
    tokens: impl IntoIterator<Item = (Token, Span)>,
    file: File,
) -> Decls {
    info!("parsing file with id {file}");

    let mut parser = Parser::new(tokens, file);
    let decls = parser.parse_program();

    driver.report(parser.msgs);

    let mut unconcretifier = Unconcretifier::new();
    let decls = unconcretifier.unconcretify(decls);

    driver.report(unconcretifier.msgs);

    trace!("done parsing file {file}");

    decls
}

#[derive(Debug)]
struct Parser<I> {
    tokens: I,
    curr: Option<(Token, Span)>,
    prev: Option<(Token, Span)>,
    msgs: Messages,
    default_span: Span,

    in_implicit: bool,
}

impl<I> Parser<I>
where
    I: Iterator<Item = (Token, Span)>,
{
    pub fn new<In>(tokens: In, file: File) -> Self
    where
        In: IntoIterator<Item = (Token, Span), IntoIter = I>,
    {
        let mut parser = Self {
            tokens: tokens.into_iter(),

            curr: None,
            prev: None,

            msgs: Messages::new(),
            default_span: Span::new(file, 0, 0),

            in_implicit: false,
        };

        parser.advance();
        parser
    }

    fn is_done(&self) -> bool {
        self.curr.is_none()
    }

    fn advance(&mut self) {
        self.prev = self.curr.take();
        if let Some(curr) = self.tokens.next() {
            self.curr = Some(curr);
        }
    }

    fn peek(&self, matcher: impl Matcher) -> bool {
        self.curr
            .as_ref()
            .map(|(tok, _)| matcher.matches(tok))
            .unwrap_or(false)
    }

    fn consume(&mut self, matcher: impl Matcher) -> bool {
        if self.peek(matcher) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn matches(&mut self, matcher: impl Matcher) -> Option<Span> {
        if self.peek(matcher) {
            self.advance();
            self.prev.as_ref().map(|(_, span)| *span)
        } else {
            None
        }
    }
}
