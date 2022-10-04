use super::tree::{Expr, ExprNode};
use super::Parser;
use crate::lex::Token;
use crate::message::Span;

impl<I> Parser<I>
where
    I: Iterator<Item = (Token, Span)>,
{
    /// ```abnf
    /// expr = lam-expr
    /// ```
    pub fn parse_expr(&mut self) -> Expr {
        self.lam_expr()
    }

    /// ```abnf
    /// small-expr = arrow-expr
    /// ```
    pub fn parse_small_expr(&mut self) -> Expr {
        self.arrow_expr()
    }

    /// ```abnf
    /// lam-expr = anno-expr ["=>" expr]
    /// ; reassoc 'f x => y' as 'f (x => y)'
    /// ```
    fn lam_expr(&mut self) -> Expr {
        let expr = self.anno_expr();
        if self.consume(Token::EqArrow) {
            let body = self.parse_expr();
            let span = expr.span + body.span;

            if let ExprNode::App(func, arg) = expr.node {
                let lam_span = arg.span + body.span;
                Expr {
                    node: ExprNode::App(
                        func,
                        Box::new(Expr {
                            node: ExprNode::Lam(arg, Box::new(body)),
                            span: lam_span,
                        }),
                    ),
                    span,
                }
            } else {
                Expr {
                    node: ExprNode::Lam(Box::new(expr), Box::new(body)),
                    span,
                }
            }
        } else {
            expr
        }
    }

    /// ```abnf
    /// anno-expr = small-expr [":" small-expr]
    /// ```
    fn anno_expr(&mut self) -> Expr {
        let expr = self.parse_small_expr();
        if self.consume(Token::Colon) {
            let anno = self.parse_small_expr();
            let span = expr.span + anno.span;

            Expr {
                node: ExprNode::Anno(Box::new(expr), Box::new(anno)),
                span,
            }
        } else {
            expr
        }
    }

    /// ```abnf
    /// arrow-expr = range-expr ["->" arrow-expr]
    /// ```
    fn arrow_expr(&mut self) -> Expr {
        let expr = self.range_expr();
        if self.consume(Token::MinArrow) {
            let op_span = self.prev.as_ref().map(|(_, span)| span).copied().unwrap();
            let other = self.arrow_expr();
            let span = expr.span + other.span;

            Expr {
                node: ExprNode::Fun(op_span, Box::new(expr), Box::new(other)),
                span,
            }
        } else {
            expr
        }
    }

    /// ```abnf
    /// range-expr = app-expr ["upto" app-expr]
    /// ```
    fn range_expr(&mut self) -> Expr {
        let expr = self.app_expr();
        if self.consume(Token::Upto) {
            let op_span = self.prev.as_ref().map(|(_, span)| span).copied().unwrap();
            let other = self.app_expr();
            let span = expr.span + other.span;

            Expr {
                node: ExprNode::Range(op_span, Box::new(expr), Box::new(other)),
                span,
            }
        } else {
            expr
        }
    }

    /// ```abnf
    /// app-expr = base-expr [app-expr]
    /// ```
    fn app_expr(&mut self) -> Expr {
        let mut expr = self.base_expr();

        while !self.is_done() && self.peek(Self::BASE_EXPR_STARTS) {
            let arg = self.base_expr();
            let span = expr.span + arg.span;

            expr = Expr {
                node: ExprNode::App(Box::new(expr), Box::new(arg)),
                span,
            }
        }

        expr
    }

    /// Tokens that may start a `base_expr`.
    const BASE_EXPR_STARTS: &'static [Token] = &[
        Token::Name(String::new()),
        Token::Number(0),
        Token::GroupOpen,
    ];

    /// ```abnf
    /// base-expr  = NAME / NUM
    /// base-expr =/ "(" expr ")"
    /// base-expr =/ "(" OP-NAME ")"
    /// ```
    fn base_expr(&mut self) -> Expr {
        self.advance();
        if let Some((tok, span)) = self.prev.take() {
            let node = match tok {
                Token::Name(name) => ExprNode::Name(name),
                Token::Number(num) => ExprNode::Int(num),
                Token::GroupOpen => {
                    let expr = if self.peek(Self::OP_NAME_STARTS) {
                        self.op_name()
                    } else {
                        self.parse_expr()
                    };
                    if !self.consume(Token::GroupClose) {
                        self.msgs.at(span).parse_unclosed_group();
                    }
                    let close_span = self
                        .prev
                        .as_ref()
                        .map(|(_, span)| span)
                        .cloned()
                        .unwrap_or(self.default_span);

                    return Expr {
                        node: ExprNode::Group(Box::new(expr)),
                        span: span + close_span,
                    };
                }

                _ => {
                    self.msgs.at(span).parse_expected_base_expr();
                    ExprNode::Invalid
                }
            };

            Expr { node, span }
        } else {
            unreachable!()
        }
    }

    /// Tokens that may start an `op_name`.
    const OP_NAME_STARTS: &'static [Token] = &[Token::MinArrow, Token::Upto];

    /// ```abnf
    /// OP-NAME = "->" / "upto"
    /// ```
    fn op_name(&mut self) -> Expr {
        self.advance();
        if let Some((tok, span)) = self.prev.take() {
            let node = match tok {
                Token::MinArrow => ExprNode::Name("->".into()),
                Token::Upto => ExprNode::Name("upto".into()),
                _ => unreachable!(),
            };

            Expr { node, span }
        } else {
            unreachable!()
        }
    }
}
