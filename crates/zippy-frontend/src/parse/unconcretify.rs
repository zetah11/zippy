//! The parser itself is fairly accepting, not distinguishing between patterns, types, or expressions; for instance
//! `(x => x) => x` gives a valid parse. The job of this module is to produce a HIR tree, validating away cases like
//! that in the process.

use zippy_common::hir::{self, BindIdGenerator};
use zippy_common::message::{Messages, Span};

use super::tree as cst;

#[derive(Debug, Default)]
pub struct Unconcretifier {
    pub msgs: Messages,
    bind_id: BindIdGenerator,
}

impl Unconcretifier {
    pub fn new() -> Self {
        Self {
            msgs: Messages::new(),
            bind_id: BindIdGenerator::new(),
        }
    }

    pub fn unconcretify(&mut self, decls: Vec<cst::Decl>) -> hir::Decls {
        self.unconc_decls(decls)
    }

    fn unconc_decls(&mut self, decls: Vec<cst::Decl>) -> hir::Decls {
        let mut values = Vec::with_capacity(decls.len());
        let mut types = Vec::new();

        for decl in decls {
            match decl.node {
                cst::DeclNode::TypeDecl { pat, bind } => {
                    let (pat, insts) = self.unconc_pat(pat);

                    if let Some(ex) = insts.first() {
                        self.msgs.at(ex.span).parse_types_take_no_implicits();
                    }

                    let anno = hir::Type {
                        node: hir::TypeNode::Wildcard,
                        span: pat.span,
                    };

                    let bind = if let Some(ty) = bind {
                        self.unconc_type(ty)
                    } else {
                        hir::Type {
                            node: hir::TypeNode::Type,
                            span: pat.span,
                        }
                    };

                    types.push(hir::TypeDef {
                        span: decl.span,
                        id: self.bind_id.fresh(),
                        anno,
                        bind,
                        pat,
                    });
                }

                cst::DeclNode::ValueDecl { pat, bind } => {
                    let (pat, insts) = self.unconc_pat(pat);
                    let implicits = self.unconc_insts(insts);

                    let anno = hir::Type {
                        node: hir::TypeNode::Wildcard,
                        span: pat.span,
                    };

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
                        id: self.bind_id.fresh(),
                        implicits,
                        pat,
                        anno,
                        bind,
                    });
                }

                cst::DeclNode::FunDecl {
                    name,
                    implicits,
                    args,
                    anno,
                    bind,
                } => {
                    let (pat, insts) = self.unconc_pat(name);

                    let mut implicits_error = false;
                    if !insts.is_empty() {
                        let span = insts.into_iter().map(|ex| ex.span).sum();
                        self.msgs.at(span).parse_disallowed_implicits();
                        implicits_error = true;
                    }

                    let implicits = implicits.into_iter().flat_map(unconc_list).collect();
                    let implicits = self.unconc_insts(implicits);

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

                    let span = bind.span + anno.span;
                    let mut bind = hir::Expr {
                        node: hir::ExprNode::Anno(Box::new(bind), anno),
                        span,
                    };

                    for arg in args.into_iter().rev() {
                        let (arg, implicits) = self.unconc_pat(arg);

                        if !implicits.is_empty() && !implicits_error {
                            let span = implicits.into_iter().map(|ex| ex.span).sum();
                            self.msgs.at(span).parse_disallowed_implicits();
                            implicits_error = true;
                        }

                        let span = bind.span + arg.span;
                        bind = hir::Expr {
                            node: hir::ExprNode::Lam(self.bind_id.fresh(), arg, Box::new(bind)),
                            span,
                        };
                    }

                    let span = pat.span;
                    values.push(hir::ValueDef {
                        span: decl.span,
                        id: self.bind_id.fresh(),
                        implicits,
                        pat,
                        anno: hir::Type {
                            node: hir::TypeNode::Wildcard,
                            span,
                        },
                        bind,
                    });
                }
            }
        }

        values.shrink_to_fit();

        hir::Decls { values, types }
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
                let (pat, insts) = self.unconc_pat(*pat);
                let body = Box::new(self.unconc_expr(*body));

                if !insts.is_empty() {
                    let span = insts.into_iter().map(|ex| ex.span).sum();
                    self.msgs.at(span).parse_generic_lambda();
                }

                hir::ExprNode::Lam(self.bind_id.fresh(), pat, body)
            }
            cst::ExprNode::App(fun, arg) => {
                let fun = Box::new(self.unconc_expr(*fun));
                let arg = Box::new(self.unconc_expr(*arg));
                hir::ExprNode::App(fun, arg)
            }
            cst::ExprNode::Inst(fun, args) => {
                let fun = Box::new(self.unconc_expr(*fun));
                let args = unconc_list(*args)
                    .into_iter()
                    .map(|arg| self.unconc_type(arg))
                    .collect();
                hir::ExprNode::Inst(fun, args)
            }
            cst::ExprNode::Anno(expr, anno) => {
                let expr = Box::new(self.unconc_expr(*expr));
                let anno = self.unconc_type(*anno);
                hir::ExprNode::Anno(expr, anno)
            }
            cst::ExprNode::Wildcard => hir::ExprNode::Hole,
            cst::ExprNode::Invalid => hir::ExprNode::Invalid,

            cst::ExprNode::Type => {
                self.msgs.at(expr.span).parse_expected_expr();
                hir::ExprNode::Invalid
            }
        };

        hir::Expr {
            node,
            span: expr.span,
        }
    }

    fn unconc_pat(&mut self, pat: cst::Expr) -> (hir::Pat, Vec<cst::Expr>) {
        let (node, insts) = match pat.node {
            cst::ExprNode::Name(name) => (hir::PatNode::Name(name), vec![]),
            cst::ExprNode::Group(pat) => return self.unconc_pat(*pat),
            cst::ExprNode::BinOp(..) => todo!("constructor patterns not yet supported"),
            cst::ExprNode::Tuple(x, y) => {
                let (x, mut insts) = self.unconc_pat(*x);
                let (y, other) = self.unconc_pat(*y);
                insts.extend(other);
                (hir::PatNode::Tuple(Box::new(x), Box::new(y)), insts)
            }
            cst::ExprNode::Anno(pat, ty) => {
                let (pat, insts) = self.unconc_pat(*pat);
                let ty = self.unconc_type(*ty);
                (hir::PatNode::Anno(Box::new(pat), ty), insts)
            }
            cst::ExprNode::Inst(pat, insts) => {
                let (pat, other) = self.unconc_pat(*pat);
                let mut insts = unconc_list(*insts);
                insts.extend(other);

                return (pat, insts);
            }
            cst::ExprNode::Wildcard => (hir::PatNode::Wildcard, vec![]),
            cst::ExprNode::Invalid => (hir::PatNode::Invalid, vec![]),
            _ => {
                self.msgs.at(pat.span).parse_not_a_pattern();
                (hir::PatNode::Invalid, vec![])
            }
        };

        (
            hir::Pat {
                node,
                span: pat.span,
            },
            insts,
        )
    }

    fn unconc_insts(&mut self, insts: Vec<cst::Expr>) -> Vec<(String, Span)> {
        insts
            .into_iter()
            .filter_map(|inst| match inst.node {
                cst::ExprNode::Name(name) => Some((name, inst.span)),
                _ => {
                    self.msgs.at(inst.span).parse_not_a_type_name();
                    None
                }
            })
            .collect()
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

            cst::ExprNode::Name(name) => hir::TypeNode::Name(name),

            cst::ExprNode::Int(v) => hir::TypeNode::Range(0, v as i64),

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

            cst::ExprNode::Type => hir::TypeNode::Type,

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
}

fn binop_to_name(op: cst::BinOp) -> &'static str {
    match op {
        cst::BinOp::Mul => "*",
    }
}

/// Turn a tuple expression like `(a, b), c, d, (e, f)` into a list of expressions like
/// `(a, b)`, `c`, `d`, `(e, f)`.
fn unconc_list(pat: cst::Expr) -> Vec<cst::Expr> {
    match pat.node {
        cst::ExprNode::Tuple(a, b) => {
            let mut list = unconc_list(*b);
            list.insert(0, *a);
            list
        }

        _ => vec![pat],
    }
}
