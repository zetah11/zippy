use std::collections::HashMap;

use crate::message::{Messages, Span};
use crate::mir::{Context, Decls, Expr, ExprNode, ExprSeq, Type, TypeId, Types, Value, ValueDef};
use crate::resolve::names::{Name, Names};
use crate::tyck::{self, UniVar};
use crate::Driver;

type HiType = tyck::Type;
type HiPat = tyck::Pat<HiType>;
type HiPatNode = tyck::PatNode<HiType>;
type HiExpr = tyck::Expr<HiType>;
type HiExprNode = tyck::ExprNode<HiType>;
type HiDecls = tyck::Decls<HiType>;
type HiCtx = tyck::Context;

pub fn lower(
    driver: &mut impl Driver,
    subst: &HashMap<UniVar, HiType>,
    names: &mut Names,
    context: HiCtx,
    decls: HiDecls,
) -> (Types, Context, Decls) {
    let mut lowerer = Lowerer::new(names, subst);

    lowerer.lower_context(context);
    let decls = lowerer.lower_decls(decls);

    driver.report(lowerer.messages);

    (lowerer.types, lowerer.context, decls)
}

#[derive(Debug)]
struct Lowerer<'a> {
    types: Types,
    names: &'a mut Names,
    subst: &'a HashMap<UniVar, HiType>,
    messages: Messages,
    context: Context,
}

impl<'a> Lowerer<'a> {
    fn new(names: &'a mut Names, subst: &'a HashMap<UniVar, HiType>) -> Self {
        Self {
            types: Types::new(),
            names,
            subst,
            messages: Messages::new(),
            context: Context::new(),
        }
    }

    fn lower_context(&mut self, context: HiCtx) {
        for (name, ty) in context {
            let ty = self.lower_type(ty);
            self.context.add(name, ty);
        }
    }

    fn lower_decls(&mut self, decls: HiDecls) -> Decls {
        let mut values = Vec::with_capacity(decls.values.len());

        for def in decls.values {
            self.destruct_binding(&mut values, def.span, def.pat, def.bind);
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

    fn destruct_binding(
        &mut self,
        within: &mut Vec<ValueDef>,
        span: Span,
        pat: HiPat,
        bind: HiExpr,
    ) {
        let bind = self.lower_expr(bind);
        match pat.node {
            HiPatNode::Invalid | HiPatNode::Wildcard => {}
            HiPatNode::Name(name) => within.push(ValueDef { span, name, bind }),

            HiPatNode::Tuple(a, b) => {
                let ty = self.lower_type(pat.data);
                let name = self.names.fresh(pat.span, None);
                self.context.add(name, ty);

                within.push(ValueDef { name, span, bind });

                let mut proj = |lowerer: &mut Lowerer<'a>, name, of, at, span, typ| {
                    let inner = lowerer.names.fresh(span, None);
                    lowerer.context.add(name, typ);
                    within.push(ValueDef {
                        name,
                        span,
                        bind: ExprSeq {
                            exprs: vec![
                                Expr {
                                    node: ExprNode::Proj {
                                        name: inner,
                                        of,
                                        at,
                                    },
                                    span,
                                    ty,
                                },
                                Expr {
                                    node: ExprNode::Produce(Value::Name(inner)),
                                    span,
                                    ty,
                                },
                            ],
                        },
                    })
                };

                println!("{:?}", self.names.get_path(&name).1);
                self.make_proj(*a, name, 0, &mut proj);
                self.make_proj(*b, name, 1, &mut proj);
            }
        }
    }

    fn destruct_expr(&mut self, within: &mut ExprSeq, pat: HiPat) -> Name {
        match pat.node {
            HiPatNode::Invalid | HiPatNode::Wildcard => self.names.fresh(pat.span, None),
            HiPatNode::Name(name) => name,
            HiPatNode::Tuple(a, b) => {
                let ty = self.lower_type(pat.data);
                let target = self.names.fresh(pat.span, None);
                self.context.add(target, ty);

                let mut proj = |_: &mut Lowerer<'a>, name, value, ndx, span, typ| {
                    within.push(Expr {
                        node: ExprNode::Proj {
                            name,
                            of: value,
                            at: ndx,
                        },
                        span,
                        ty: typ,
                    });
                };

                self.make_proj(*a, target, 0, &mut proj);
                self.make_proj(*b, target, 1, &mut proj);

                target
            }
        }
    }

    fn make_proj<F>(&mut self, pat: HiPat, value: Name, ndx: usize, proj: &mut F)
    where
        F: FnMut(&mut Lowerer<'a>, Name, Name, usize, Span, TypeId),
    {
        match pat.node {
            HiPatNode::Invalid | HiPatNode::Wildcard => {}
            HiPatNode::Name(name) => {
                let typ = self.lower_type(pat.data);
                proj(self, name, value, ndx, pat.span, typ);
            }

            HiPatNode::Tuple(a, b) => {
                let ty = self.lower_type(pat.data);
                let projd = self.names.fresh(pat.span, None);
                self.context.add(projd, ty);

                proj(self, projd, value, ndx, pat.span, ty);

                self.make_proj(*a, projd, 0, proj);
                self.make_proj(*b, projd, 1, proj);
            }
        }
    }

    fn lower_expr(&mut self, ex: HiExpr) -> ExprSeq {
        let mut seq = ExprSeq::default();
        let typ = self.lower_type(ex.data.clone());
        let (value, span) = self.make_expr(&mut seq, ex);
        seq.push(Expr {
            node: ExprNode::Produce(value),
            span,
            ty: typ,
        });
        seq
    }

    fn make_expr(&mut self, within: &mut ExprSeq, ex: HiExpr) -> (Value, Span) {
        let value = match ex.node {
            HiExprNode::Int(i) => Value::Int(i),
            HiExprNode::Name(name) => Value::Name(name),
            HiExprNode::Lam(param, body) => {
                let mut bodys = ExprSeq::default();

                let param = self.destruct_expr(&mut bodys, param);
                bodys.extend(self.lower_expr(*body).exprs);

                let body = bodys;

                let ty = self.lower_type(ex.data);

                let name = self.names.fresh(ex.span, None);
                self.context.add(name, ty);

                within.push(Expr {
                    node: ExprNode::Function { name, param, body },
                    span: ex.span,
                    ty,
                });

                Value::Name(name)
            }

            HiExprNode::App(fun, arg) => {
                let fun = if let (Value::Name(name), _) = self.make_expr(within, *fun) {
                    name
                } else {
                    unreachable!()
                };

                let (arg, _) = self.make_expr(within, *arg);

                let ty = self.lower_type(ex.data);

                let name = self.names.fresh(ex.span, None);
                self.context.add(name, ty);

                within.push(Expr {
                    node: ExprNode::Apply { name, fun, arg },
                    span: ex.span,
                    ty,
                });

                Value::Name(name)
            }

            HiExprNode::Tuple(t, u) => {
                let (t, _) = self.make_expr(within, *t);
                let (u, _) = self.make_expr(within, *u);

                let ty = self.lower_type(ex.data);

                let name = self.names.fresh(ex.span, None);
                self.context.add(name, ty);

                within.push(Expr {
                    node: ExprNode::Tuple {
                        name,
                        values: vec![t, u],
                    },
                    span: ex.span,
                    ty,
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
