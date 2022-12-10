use zippy_common::message::Span;

use super::tree::{BinOp, Expr, ExprNode};
use super::Parser;
use crate::lex::Token;

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
        self.tuple_expr()
    }

    /// ```abnf
    /// lam-expr = small-expr ["=>" expr]
    /// ; reassoc 'f x => y' as 'f (x => y)'
    /// ```
    fn lam_expr(&mut self) -> Expr {
        let expr = self.parse_small_expr();
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
    /// tuple-expr = anno-expr *("," anno-expr)
    /// ```
    fn tuple_expr(&mut self) -> Expr {
        let mut expr = self.anno_expr();
        while self.consume(Token::Comma) {
            let other = self.anno_expr();
            let span = expr.span + other.span;
            expr = Expr {
                node: ExprNode::Tuple(Box::new(expr), Box::new(other)),
                span,
            };
        }
        expr
    }

    /// ```abnf
    /// anno-expr = arrow-expr [":" arrow-expr]
    /// ```
    fn anno_expr(&mut self) -> Expr {
        let expr = self.arrow_expr();
        if self.consume(Token::Colon) {
            let anno = self.arrow_expr();
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
    /// range-expr = mul-expr ["upto" mul-expr]
    /// ```
    fn range_expr(&mut self) -> Expr {
        let expr = self.mul_expr();
        if let Some(op_span) = self.matches(Token::Upto) {
            let other = self.mul_expr();
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
    /// mul-expr = app-expr *("*" app-expr)
    /// ```
    fn mul_expr(&mut self) -> Expr {
        let mut expr = self.app_expr();

        while let Some(op_span) = self.matches(Token::Star) {
            let other = self.app_expr();
            let span = expr.span + other.span;

            expr = Expr {
                node: ExprNode::BinOp(op_span, BinOp::Mul, Box::new(expr), Box::new(other)),
                span,
            };
        }

        expr
    }

    fn is_arg(&self) -> bool {
        !self.is_done()
            && (self.peek(Self::BASE_EXPR_STARTS) || (!self.in_implicit && self.peek(Token::Pipe)))
    }

    /// ```abnf
    /// app-expr = base-expr *(app-expr / "|" small-expr "|")
    /// ```
    fn app_expr(&mut self) -> Expr {
        let mut expr = self.parse_base_expr();

        while self.is_arg() {
            if let Some(opener) = self.matches(Token::Pipe) {
                self.in_implicit = true;
                let arg = self.parse_small_expr();
                self.in_implicit = false;

                if !self.consume(Token::Pipe) {
                    self.msgs.at(opener).parse_unclosed_implicits();
                }

                let span = expr.span + arg.span;
                expr = Expr {
                    node: ExprNode::Inst(Box::new(expr), Box::new(arg)),
                    span,
                };

                continue;
            }

            let arg = self.parse_base_expr();
            let span = expr.span + arg.span;

            expr = Expr {
                node: ExprNode::App(Box::new(expr), Box::new(arg)),
                span,
            };
        }

        expr
    }

    /// Tokens that may start a `base_expr`. Does not contain [`Token::Invalid`]
    /// in order to prevent `app_expr` from consuming too many invalid tokens.
    pub const BASE_EXPR_STARTS: &'static [Token] = &[
        Token::Name(String::new()),
        Token::Number(0),
        Token::Question,
        Token::GroupOpen,
        Token::Type,
    ];

    /// ```abnf
    /// base-expr  = NAME / NUM / WILDCARD
    /// base-expr =/ "(" expr ")"
    /// base-expr =/ "(" OP-NAME ")"
    /// ```
    pub fn parse_base_expr(&mut self) -> Expr {
        self.advance();
        if let Some((tok, span)) = self.prev.take() {
            let node = match tok {
                Token::Name(name) => ExprNode::Name(name),
                Token::Number(num) => ExprNode::Int(num),
                Token::Question => ExprNode::Wildcard,
                Token::Type => ExprNode::Type,
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

                Token::Invalid => ExprNode::Invalid,

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
    const OP_NAME_STARTS: &'static [Token] = &[Token::MinArrow, Token::Upto, Token::Star];

    /// ```abnf
    /// OP-NAME = "->" / "upto"
    /// ```
    fn op_name(&mut self) -> Expr {
        self.advance();
        if let Some((tok, span)) = self.prev.take() {
            let node = match tok {
                Token::MinArrow => ExprNode::Name("->".into()),
                Token::Upto => ExprNode::Name("upto".into()),
                Token::Star => ExprNode::Name("*".into()),
                _ => unreachable!(),
            };

            Expr { node, span }
        } else {
            unreachable!()
        }
    }
}
