//! The parser itself is fairly accepting, not distinguishing between patterns, types, or expressions; for instance
//! `(x => x) => x` gives a valid parse. The job of this module is to produce a HIR tree, validating away cases like
//! that in the process.

use super::tree as cst;
use crate::hir::{self, BindId};
use crate::message::Messages;

#[derive(Debug, Default)]
pub struct Unconcretifier {
    pub msgs: Messages,
    bind_id: usize,
}

impl Unconcretifier {
    pub fn new() -> Self {
        Self {
            msgs: Messages::new(),
            bind_id: 0,
        }
    }

    pub fn unconcretify(&mut self, decls: Vec<cst::Decl>) -> hir::Decls {
        self.unconc_decls(decls)
    }

    fn unconc_decls(&mut self, decls: Vec<cst::Decl>) -> hir::Decls {
        let mut values = Vec::with_capacity(decls.len());

        for decl in decls {
            match decl.node {
                cst::DeclNode::ValueDecl { pat, anno, bind } => {
                    let pat = self.unconc_pat(pat);
                    let anno =
                        anno.map(|anno| self.unconc_type(anno))
                            .unwrap_or_else(|| hir::Type {
                                node: hir::TypeNode::Wildcard,
                                span: pat.span,
                            });

                    let bind = if let Some(bind) = bind {
                        self.unconc_expr(bind)
                    } else {
                        hir::Expr {
                            node: hir::ExprNode::Invalid,
                            span: decl.span,
                        }
                    };

                    values.push(hir::ValueDef {
                        span: decl.span,
                        id: self.fresh_bind_id(),
                        pat,
                        anno,
                        bind,
                    });
                }
            }
        }

        hir::Decls { values }
    }

    fn unconc_expr(&mut self, expr: cst::Expr) -> hir::Expr {
        let node = match expr.node {
            cst::ExprNode::Name(name) => hir::ExprNode::Name(name),
            cst::ExprNode::Int(i) => hir::ExprNode::Int(i as i64), // uhhh
            cst::ExprNode::Group(expr) => return self.unconc_expr(*expr),
            cst::ExprNode::Range(span, lo, hi) => {
                let lo = Box::new(self.unconc_expr(*lo));
                let hi = Box::new(self.unconc_expr(*hi));
                let fun = hir::Expr {
                    node: hir::ExprNode::Name("upto".into()),
                    span,
                };

                let span = lo.span + span;
                let fun = hir::Expr {
                    node: hir::ExprNode::App(Box::new(fun), lo),
                    span,
                };

                hir::ExprNode::App(Box::new(fun), hi)
            }
            cst::ExprNode::Fun(span, t, u) => {
                let t = Box::new(self.unconc_expr(*t));
                let u = Box::new(self.unconc_expr(*u));

                let fun = hir::Expr {
                    node: hir::ExprNode::Name("->".into()),
                    span,
                };

                let span = t.span + span;
                let fun = hir::Expr {
                    node: hir::ExprNode::App(Box::new(fun), t),
                    span,
                };

                hir::ExprNode::App(Box::new(fun), u)
            }
            cst::ExprNode::BinOp(span, op, x, y) => {
                let x = Box::new(self.unconc_expr(*x));
                let y = Box::new(self.unconc_expr(*y));

                let fun = hir::Expr {
                    node: hir::ExprNode::Name(binop_to_name(op).into()),
                    span,
                };

                let span = x.span + span;
                let fun = hir::Expr {
                    node: hir::ExprNode::App(Box::new(fun), x),
                    span,
                };

                hir::ExprNode::App(Box::new(fun), y)
            }
            cst::ExprNode::Tuple(x, y) => {
                let x = Box::new(self.unconc_expr(*x));
                let y = Box::new(self.unconc_expr(*y));
                hir::ExprNode::Tuple(x, y)
            }
            cst::ExprNode::Lam(pat, body) => {
                let pat = self.unconc_pat(*pat);
                let body = Box::new(self.unconc_expr(*body));
                hir::ExprNode::Lam(self.fresh_bind_id(), pat, body)
            }
            cst::ExprNode::App(fun, arg) => {
                let fun = Box::new(self.unconc_expr(*fun));
                let arg = Box::new(self.unconc_expr(*arg));
                hir::ExprNode::App(fun, arg)
            }
            cst::ExprNode::Anno(expr, anno) => {
                let expr = Box::new(self.unconc_expr(*expr));
                let anno = self.unconc_type(*anno);
                hir::ExprNode::Anno(expr, anno)
            }
            cst::ExprNode::Wildcard => hir::ExprNode::Hole,
            cst::ExprNode::Invalid => hir::ExprNode::Invalid,
        };

        hir::Expr {
            node,
            span: expr.span,
        }
    }

    fn unconc_pat(&mut self, pat: cst::Expr) -> hir::Pat {
        let node = match pat.node {
            cst::ExprNode::Name(name) => hir::PatNode::Name(name),
            cst::ExprNode::Group(pat) => return self.unconc_pat(*pat),
            cst::ExprNode::BinOp(..) => todo!("constructor patterns not yet supported"),
            cst::ExprNode::Tuple(x, y) => {
                let x = Box::new(self.unconc_pat(*x));
                let y = Box::new(self.unconc_pat(*y));
                hir::PatNode::Tuple(x, y)
            }
            cst::ExprNode::Wildcard => hir::PatNode::Wildcard,
            cst::ExprNode::Invalid => hir::PatNode::Invalid,
            _ => {
                self.msgs.at(pat.span).parse_not_a_pattern();
                hir::PatNode::Invalid
            }
        };

        hir::Pat {
            node,
            span: pat.span,
        }
    }

    fn unconc_type(&mut self, typ: cst::Expr) -> hir::Type {
        let node = match typ.node {
            cst::ExprNode::Range(_, lo, hi) => {
                let lo = self.unconc_expr(*lo);
                let hi = self.unconc_expr(*hi);

                match (lo.node, hi.node) {
                    (hir::ExprNode::Int(lo), hir::ExprNode::Int(hi)) => {
                        hir::TypeNode::Range(lo, hi)
                    }

                    (hir::ExprNode::Int(_), _) => {
                        self.msgs.at(hi.span).parse_range_not_an_int();
                        hir::TypeNode::Invalid
                    }

                    (_, hir::ExprNode::Int(_)) => {
                        self.msgs.at(lo.span).parse_range_not_an_int();
                        hir::TypeNode::Invalid
                    }

                    _ => {
                        self.msgs.at(lo.span + hi.span).parse_range_not_an_int();
                        hir::TypeNode::Invalid
                    }
                }
            }

            cst::ExprNode::Fun(_, t, u) => {
                let t = Box::new(self.unconc_type(*t));
                let u = Box::new(self.unconc_type(*u));
                hir::TypeNode::Fun(t, u)
            }

            cst::ExprNode::BinOp(_, cst::BinOp::Mul, t, u) => {
                let t = Box::new(self.unconc_type(*t));
                let u = Box::new(self.unconc_type(*u));
                hir::TypeNode::Prod(t, u)
            }

            cst::ExprNode::Group(typ) => return self.unconc_type(*typ),

            cst::ExprNode::Wildcard => hir::TypeNode::Wildcard,

            _ => {
                self.msgs.at(typ.span).parse_not_a_type();
                hir::TypeNode::Invalid
            }
        };

        hir::Type {
            node,
            span: typ.span,
        }
    }

    fn fresh_bind_id(&mut self) -> BindId {
        let id = BindId(self.bind_id);
        self.bind_id += 1;
        id
    }
}

fn binop_to_name(op: cst::BinOp) -> &'static str {
    match op {
        cst::BinOp::Mul => "*",
    }
}
