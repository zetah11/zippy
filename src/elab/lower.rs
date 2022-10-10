use std::collections::HashMap;

use crate::message::{Messages, Span};
use crate::mir::{
    Decls, Expr, ExprNode, ExprSeq, Pat, PatNode, Type, TypeId, Types, Value, ValueDef,
};
use crate::resolve::names::{Name, Names};
use crate::tyck::{self, UniVar};
use crate::Driver;

type HiType = tyck::Type;
type HiPat = tyck::Pat<HiType>;
type HiPatNode = tyck::PatNode<HiType>;
type HiExpr = tyck::Expr<HiType>;
type HiExprNode = tyck::ExprNode<HiType>;
type HiDecls = tyck::Decls<HiType>;

pub fn lower(
    driver: &mut impl Driver,
    subst: &HashMap<UniVar, HiType>,
    names: &mut Names,
    decls: HiDecls,
) -> (Types, Decls) {
    let mut lowerer = Lowerer::new(names, subst);
    let decls = lowerer.lower_decls(decls);

    driver.report(lowerer.messages);

    (lowerer.types, decls)
}

#[derive(Debug)]
struct Lowerer<'a> {
    types: Types,
    names: &'a mut Names,
    subst: &'a HashMap<UniVar, HiType>,
    messages: Messages,
}

impl<'a> Lowerer<'a> {
    fn new(names: &'a mut Names, subst: &'a HashMap<UniVar, HiType>) -> Self {
        Self {
            types: Types::new(),
            names,
            subst,
            messages: Messages::new(),
        }
    }

    fn lower_decls(&mut self, decls: HiDecls) -> Decls {
        let mut values = Vec::with_capacity(decls.values.len());

        for def in decls.values {
            values.extend(self.destruct_binding(def.span, def.pat, def.bind));
        }

        Decls { values }
    }

    fn lower_type(&mut self, ty: HiType) -> TypeId {
        match ty {
            HiType::Fun(t, u) => {
                let t = self.lower_type(*t);
                let u = self.lower_type(*u);

                self.types.add(Type::Fun(t, u))
            }

            HiType::Product(t, u) => {
                let t = self.lower_type(*t);
                let u = self.lower_type(*u);

                self.types.add(Type::Product(t, u))
            }

            HiType::Range(lo, hi) => self.types.add(Type::Range(lo, hi)),

            HiType::Var(v) => {
                if let Some(ty) = self.subst.get(&v).cloned() {
                    self.lower_type(ty)
                } else {
                    unimplemented!()
                }
            }

            HiType::Invalid | HiType::Number => unreachable!(),
        }
    }

    fn destruct_binding(&mut self, span: Span, pat: HiPat, bind: HiExpr) -> Vec<ValueDef> {
        let bind = self.lower_expr(bind);
        match pat.node {
            HiPatNode::Invalid | HiPatNode::Wildcard => vec![],
            HiPatNode::Name(name) => vec![ValueDef { span, name, bind }],

            HiPatNode::Tuple(a, b) => {
                let name = self.names.fresh(pat.span, None);
                let mut defs = vec![ValueDef { span, name, bind }];

                defs.extend(self.make_proj_binding(*a, name, 0));
                defs.extend(self.make_proj_binding(*b, name, 1));

                defs
            }
        }
    }

    fn make_proj_binding(&mut self, pat: HiPat, value: Name, ndx: usize) -> Vec<ValueDef> {
        match pat.node {
            HiPatNode::Invalid | HiPatNode::Wildcard => vec![],
            HiPatNode::Name(name) => {
                let projd = self.names.fresh(pat.span, None);
                let typ = self.lower_type(pat.data);
                let exprs = vec![
                    Expr {
                        node: ExprNode::Proj {
                            name: projd,
                            of: value,
                            at: ndx,
                        },
                        span: pat.span,
                        typ,
                    },
                    Expr {
                        node: ExprNode::Produce(Value::Name(projd)),
                        span: pat.span,
                        typ,
                    },
                ];

                vec![ValueDef {
                    name,
                    bind: ExprSeq { exprs },
                    span: pat.span,
                }]
            }
            HiPatNode::Tuple(a, b) => {
                let inner = self.names.fresh(pat.span, None);
                let projd = self.names.fresh(pat.span, None);
                let typ = self.lower_type(pat.data);
                let mut defs = vec![ValueDef {
                    name: projd,
                    span: pat.span,
                    bind: ExprSeq {
                        exprs: vec![
                            Expr {
                                node: ExprNode::Proj {
                                    name: inner,
                                    of: value,
                                    at: ndx,
                                },
                                span: pat.span,
                                typ,
                            },
                            Expr {
                                node: ExprNode::Produce(Value::Name(inner)),
                                span: pat.span,
                                typ,
                            },
                        ],
                    },
                }];

                defs.extend(self.make_proj_binding(*a, projd, 0));
                defs.extend(self.make_proj_binding(*b, projd, 1));

                defs
            }
        }
    }

    fn destruct_expr(&mut self, within: &mut ExprSeq, pat: HiPat) -> Name {
        match pat.node {
            HiPatNode::Invalid | HiPatNode::Wildcard => self.names.fresh(pat.span, None),
            HiPatNode::Name(name) => name,
            HiPatNode::Tuple(a, b) => {
                let target = self.names.fresh(pat.span, None);

                (self.make_proj_expr(within, *a, target, 0));
                (self.make_proj_expr(within, *b, target, 1));

                target
            }
        }
    }

    fn make_proj_expr(&mut self, within: &mut ExprSeq, pat: HiPat, value: Name, ndx: usize) {
        match pat.node {
            HiPatNode::Invalid | HiPatNode::Wildcard => {}
            HiPatNode::Name(name) => {
                let typ = self.lower_type(pat.data);
                within.push(Expr {
                    node: ExprNode::Proj {
                        name,
                        of: value,
                        at: ndx,
                    },
                    span: pat.span,
                    typ,
                })
            }

            HiPatNode::Tuple(a, b) => {
                let projd = self.names.fresh(pat.span, None);
                let typ = self.lower_type(pat.data);
                within.push(Expr {
                    node: ExprNode::Proj {
                        name: projd,
                        of: value,
                        at: ndx,
                    },
                    span: pat.span,
                    typ,
                });

                (self.make_proj_expr(within, *a, projd, 0));
                (self.make_proj_expr(within, *b, projd, 1));
            }
        }
    }

    fn lower_pat(&mut self, pat: HiPat) -> Pat {
        let node = match pat.node {
            HiPatNode::Name(name) => PatNode::Name(name),
            HiPatNode::Wildcard => PatNode::Wildcard,
            HiPatNode::Invalid => PatNode::Invalid,

            HiPatNode::Tuple(..) => todo!(),
        };

        Pat {
            node,
            span: pat.span,
            typ: self.lower_type(pat.data),
        }
    }

    fn lower_expr(&mut self, ex: HiExpr) -> ExprSeq {
        let mut seq = ExprSeq::default();
        let typ = self.lower_type(ex.data.clone());
        let (value, span) = self.go(&mut seq, ex);
        seq.push(Expr {
            node: ExprNode::Produce(value),
            span,
            typ,
        });
        seq
    }

    fn go(&mut self, within: &mut ExprSeq, ex: HiExpr) -> (Value, Span) {
        let value = match ex.node {
            HiExprNode::Int(i) => Value::Int(i),
            HiExprNode::Name(name) => Value::Name(name),
            HiExprNode::Lam(param, body) => {
                let mut bodys = ExprSeq::default();

                let param = self.destruct_expr(&mut bodys, param);
                bodys.extend(self.lower_expr(*body).exprs);

                let body = bodys;

                let typ = self.lower_type(ex.data);

                let name = self.names.fresh(ex.span, None);

                within.push(Expr {
                    node: ExprNode::Function { name, param, body },
                    span: ex.span,
                    typ,
                });

                Value::Name(name)
            }

            HiExprNode::App(fun, arg) => {
                let fun = if let (Value::Name(name), _) = self.go(within, *fun) {
                    name
                } else {
                    unreachable!()
                };

                let (arg, _) = self.go(within, *arg);

                let typ = self.lower_type(ex.data);

                let name = self.names.fresh(ex.span, None);
                within.push(Expr {
                    node: ExprNode::Apply { name, fun, arg },
                    span: ex.span,
                    typ,
                });

                Value::Name(name)
            }

            HiExprNode::Tuple(t, u) => {
                let (t, _) = self.go(within, *t);
                let (u, _) = self.go(within, *u);

                let typ = self.lower_type(ex.data);

                let name = self.names.fresh(ex.span, None);
                within.push(Expr {
                    node: ExprNode::Tuple {
                        name,
                        values: vec![t, u],
                    },
                    span: ex.span,
                    typ,
                });

                Value::Name(name)
            }

            HiExprNode::Hole => {
                self.messages
                    .at(ex.span)
                    .elab_report_hole(format!("{:?}", ex.data));
                Value::Invalid
            }

            HiExprNode::Invalid => Value::Invalid,

            HiExprNode::Anno(..) => unreachable!(),
        };

        (value, ex.span)
    }
}
