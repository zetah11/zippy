use zippy_common::source::Span;

use super::Parser;
use crate::messages::ParseMessages;
use crate::parser::cst::{Item, ItemNode};
use crate::parser::tokens::{Token, TokenType};

impl<I: Iterator<Item = Token>> Parser<'_, I> {
    /// Parse an expression.
    pub fn parse_expr(&mut self) -> Item {
        if let Some(opener) = self.consume(TokenType::Entry) {
            self.entry(opener)
        } else {
            self.annotation_expr()
        }
    }

    /// Returns `true` if the current token could be the start of an expression.
    pub(super) fn peek_expr(&self) -> bool {
        const EXPR_STARTS: &[TokenType] = &[TokenType::Entry];

        self.peek(EXPR_STARTS).is_some() || self.peek_simple_expr()
    }

    fn entry(&mut self, opener: Span) -> Item {
        let inner = if self.peek_item() {
            Box::new(self.parse_item())
        } else {
            Box::new(Item {
                span: opener,
                node: ItemNode::Group(Vec::new()),
            })
        };

        Item {
            span: opener,
            node: ItemNode::Entry(inner),
        }
    }

    fn annotation_expr(&mut self) -> Item {
        let expr = self.invoke_expr();

        if self.consume(TokenType::Colon).is_some() {
            let ty = Box::new(self.invoke_expr());
            Item {
                span: expr.span + ty.span,
                node: ItemNode::Annotation(Box::new(expr), ty),
            }
        } else {
            expr
        }
    }

    fn invoke_expr(&mut self) -> Item {
        let mut expr = self.simple_expr();

        while let Some(period) = self.consume(TokenType::Period) {
            if self.peek_simple_expr() {
                let field = self.simple_expr();
                expr = Item {
                    span: expr.span + field.span,
                    node: ItemNode::Path(Box::new(expr), Box::new(field)),
                };
            } else {
                self.at(period).expected_name();
            }
        }

        expr
    }

    fn peek_simple_expr(&self) -> bool {
        const SIMPLE_STARTS: &[TokenType] = &[
            TokenType::Name(String::new()),
            TokenType::Number(String::new()),
            TokenType::String(String::new()),
            TokenType::LeftParen,
            TokenType::Indent,
        ];

        self.peek(SIMPLE_STARTS).is_some()
    }

    fn simple_expr(&mut self) -> Item {
        let current = self.current.take();
        self.advance();

        if let Some(Token { span, .. }) = current {
            self.previous = Some(span);
        }

        let (node, span) = match current {
            Some(Token {
                kind: TokenType::Name(name),
                span,
                ..
            }) => (ItemNode::Name(name), span),

            Some(Token {
                kind: TokenType::Number(number),
                span,
                ..
            }) => (ItemNode::Number(number), span),

            Some(Token {
                kind: TokenType::String(string),
                span,
                ..
            }) => (ItemNode::String(string), span),

            Some(Token {
                kind: TokenType::LeftParen,
                span: opener,
                ..
            }) => {
                let items = self.parse_items();

                let span = if let Some(span) = self.consume(TokenType::RightParen) {
                    span
                } else {
                    self.at(opener).unclosed_parenthesis();
                    self.closest_span()
                };

                (ItemNode::Group(items), opener + span)
            }

            Some(Token {
                kind: TokenType::Indent,
                span: opener,
                ..
            }) => {
                let items = self.parse_items();
                let dedent_span = self.closest_span();

                if self.skip_until(TokenType::Dedent) {
                    self.at(dedent_span).expected_item();
                }

                let _ = self
                    .consume(TokenType::Dedent)
                    .expect("lexer should not produce more indents than dedents");

                (ItemNode::Group(items), opener + dedent_span)
            }

            _ => {
                let span = self.closest_span();
                self.at(span).expected_expression();
                (ItemNode::Invalid, span)
            }
        };

        Item { span, node }
    }
}
