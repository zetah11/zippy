use common::message::Span;

use super::tree::{Decl, DeclNode};
use super::Parser;
use crate::lex::Token;

impl<I> Parser<I>
where
    I: Iterator<Item = (Token, Span)>,
{
    /// ```abnf
    /// prog = decls
    /// ; and nothing else
    /// ```
    pub fn parse_program(&mut self) -> Vec<Decl> {
        let mut decls = vec![];

        while !self.is_done() {
            decls.extend(self.parse_decls());

            let mut opener_span = None;
            while !self.is_done() && !self.peek(Self::DECL_STARTS) {
                self.advance();
                if opener_span.is_none() {
                    opener_span = self.prev.as_ref().map(|(_, span)| *span);
                }
            }

            if let Some(span) = opener_span {
                self.msgs.at(span).parse_expected_declaration();
            }
        }

        decls
    }

    /// ```abnf
    /// decls  = [decl *(";" decl) [";"]]
    /// decls =/ "(" decls ")"
    /// ```
    pub fn parse_decls(&mut self) -> Vec<Decl> {
        if let Some(span) = self.matches(Token::GroupOpen) {
            let decls = self.parse_decls();
            if !self.consume(Token::GroupClose) {
                self.msgs.at(span).parse_unclosed_group();
            }

            decls
        } else if self.peek(Self::DECL_STARTS) {
            let mut decls = vec![self.decl()];
            while self.consume(Token::Delimit) {
                if !self.peek(Self::DECL_STARTS) {
                    break;
                }
                decls.push(self.decl());
            }
            decls
        } else {
            vec![]
        }
    }

    /// Tokens that may start a `decl`.
    const DECL_STARTS: &'static [Token] = &[Token::GroupOpen, Token::Let];

    /// ```abnf
    /// decl = let-decl
    /// ```
    fn decl(&mut self) -> Decl {
        if let Some(span) = self.matches(Token::Let) {
            self.let_decl(span)
        } else {
            unreachable!()
        }
    }

    /// ```abnf
    /// let-decl = "let" small-expr [":" small-expr] ["=" expr]
    /// ```
    fn let_decl(&mut self, let_span: Span) -> Decl {
        let pat = self.parse_small_expr();

        let anno = self.consume(Token::Colon).then(|| self.parse_small_expr());
        let bind = self.consume(Token::Equal).then(|| self.parse_expr());

        let span = bind
            .as_ref()
            .map(|bind| bind.span)
            .or_else(|| anno.as_ref().map(|anno| anno.span))
            .unwrap_or(pat.span);

        Decl {
            node: DeclNode::ValueDecl { pat, anno, bind },
            span: let_span + span,
        }
    }
}
