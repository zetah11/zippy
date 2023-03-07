use zippy_common::source::Span;

use super::Parser;
use crate::messages::ParseMessages;
use crate::parser::cst::{Item, ItemNode};
use crate::parser::tokens::{Token, TokenType};

impl<I: Iterator<Item = Token>> Parser<'_, I> {
    /// Parse several delimited items.
    pub fn parse_items(&mut self) -> Vec<Item> {
        let mut items = Vec::new();

        while self
            .consume(&[TokenType::Eol, TokenType::Semicolon])
            .is_some()
        {
            // pass
        }

        while self.peek_item() {
            items.push(self.parse_item());

            while self
                .consume(&[TokenType::Eol, TokenType::Semicolon])
                .is_some()
            {
                // pass
            }
        }

        items
    }

    /// Parse a single item.
    pub fn parse_item(&mut self) -> Item {
        if let Some(opener) = self.consume(TokenType::Import) {
            self.parse_import(opener)
        } else if let Some(opener) = self.consume(TokenType::Let) {
            self.parse_let(opener)
        } else if self.peek_expr() {
            self.parse_expr()
        } else {
            let span = self.closest_span();
            self.at(span).expected_item();
            Item {
                span,
                node: ItemNode::Invalid,
            }
        }
    }

    /// Returns `true` if the next token could be the start of an item.
    pub(super) fn peek_item(&self) -> bool {
        const ITEM_STARTS: &[TokenType] = &[TokenType::Import, TokenType::Let];
        self.peek(ITEM_STARTS).is_some() || self.peek_expr()
    }

    /// Parse an `import` item.
    fn parse_import(&mut self, opener: Span) -> Item {
        if !self.peek_expr() {
            let span = opener;
            self.at(span).expected_name();
            return Item {
                span,
                node: ItemNode::Invalid,
            };
        }

        let item = self.parse_expr();
        Item {
            span: opener + item.span,
            node: ItemNode::Import(Box::new(item)),
        }
    }

    /// Parse a `let` binding
    fn parse_let(&mut self, opener: Span) -> Item {
        if !self.peek_expr() {
            let span = opener;
            self.at(span).expected_pattern();
            return Item {
                span,
                node: ItemNode::Invalid,
            };
        }

        let pattern = Box::new(self.parse_expr());

        let body = self
            .consume(TokenType::Equal)
            .map(|span| {
                if self.peek_expr() {
                    self.parse_expr()
                } else {
                    self.at(span).expected_expression();
                    Item {
                        span,
                        node: ItemNode::Invalid,
                    }
                }
            })
            .map(Box::new);

        Item {
            span: opener + self.closest_span(),
            node: ItemNode::Let { pattern, body },
        }
    }
}
