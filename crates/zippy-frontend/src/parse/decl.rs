use zippy_common::message::Span;

use super::tree::{Decl, DeclNode, Expr, ExprNode};
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
    const DECL_STARTS: &'static [Token] = &[Token::GroupOpen, Token::Fun, Token::Let];

    /// ```abnf
    /// decl = let-decl / fun-decl
    /// ```
    fn decl(&mut self) -> Decl {
        if let Some(span) = self.matches(Token::Let) {
            self.let_decl(span)
        } else if let Some(span) = self.matches(Token::Fun) {
            self.fun_decl(span)
        } else {
            unreachable!()
        }
    }

    /// ```abnf
    /// let-decl = "let" small-expr ["=" expr]
    /// ```
    fn let_decl(&mut self, let_span: Span) -> Decl {
        let pat = self.parse_small_expr();
        let bind = self.consume(Token::Equal).then(|| self.parse_expr());

        let span = bind.as_ref().map(|bind| bind.span).unwrap_or(pat.span);
        let anno_span = pat.span;

        Decl {
            node: DeclNode::ValueDecl {
                pat,
                anno: Some(Expr {
                    node: ExprNode::Wildcard,
                    span: anno_span,
                }),
                bind,
            },
            span: let_span + span,
        }
    }

    /// ```abnf
    /// fun-decl = "fun" base-expr ["|" small-expr "|"] *(base-expr) [":" small-expr] ["=" expr]
    /// ```
    fn fun_decl(&mut self, fun_span: Span) -> Decl {
        let name = self.parse_base_expr();

        let implicits = if let Some(opener) = self.matches(Token::Pipe) {
            self.in_implicit = true;
            let args = self.parse_small_expr();
            self.in_implicit = false;

            if !self.consume(Token::Pipe) {
                self.msgs.at(opener).parse_unclosed_implicits();
            }

            Some(args)
        } else {
            None
        };

        let mut args = Vec::new();
        while !self.is_done() && self.peek(Self::BASE_EXPR_STARTS) {
            let arg = self.parse_base_expr();
            args.push(arg);
        }

        let anno = self.consume(Token::Colon).then(|| self.parse_small_expr());
        let bind = self.consume(Token::Equal).then(|| self.parse_expr());

        let span = bind
            .as_ref()
            .map(|bind| bind.span)
            .or_else(|| anno.as_ref().map(|anno| anno.span))
            .or_else(|| args.iter().map(|arg| arg.span).reduce(|a, b| a + b))
            .unwrap_or(name.span);

        Decl {
            node: DeclNode::FunDecl {
                name,
                implicits,
                args,
                anno,
                bind,
            },
            span: fun_span + span,
        }
    }
}
