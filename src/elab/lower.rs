use std::collections::HashMap;

use log::{debug, trace};

use crate::message::{Messages, Span};
use crate::mir::pretty::Prettier;
use crate::mir::{
    Branch, BranchNode, Context, Decls, Expr, ExprNode, ExprSeq, Type, TypeId, Types, Value,
    ValueDef, ValueNode,
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
type HiCtx = tyck::Context;

pub fn lower(
    driver: &mut impl Driver,
    subst: &HashMap<UniVar, HiType>,
    names: &mut Names,
    context: HiCtx,
    decls: HiDecls,
) -> (Types, Context, Decls) {
    debug!("beginning lowering");
    let mut lowerer = Lowerer::new(names, subst);

    lowerer.lower_context(context);
    let decls = lowerer.lower_decls(decls);

    driver.report(lowerer.messages);

    trace!("done lowering");

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

        Decls::new(values)
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

            HiType::Invalid => self.types.add(Type::Invalid),
            HiType::Number => unreachable!(),
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

                let mut proj = |lowerer: &mut Lowerer<'a>, name, of, at, span, ty| {
                    let inner = lowerer.names.fresh(span, None);
                    lowerer.context.add(name, ty);
                    within.push(ValueDef {
                        name,
                        span,
                        bind: ExprSeq {
                            span,
                            ty,
                            exprs: vec![Expr {
                                node: ExprNode::Proj {
                                    name: inner,
                                    of,
                                    at,
                                },
                                span,
                                ty,
                            }],
                            branch: Branch {
                                node: BranchNode::Return(Value {
                                    node: ValueNode::Name(inner),
                                    span,
                                    ty,
                                }),
                                span,
                                ty,
                            },
                        },
                    })
                };

                println!("{:?}", self.names.get_path(&name).1);
                self.make_proj(*a, name, 0, &mut proj);
                self.make_proj(*b, name, 1, &mut proj);
            }
        }
    }

    fn destruct_expr(&mut self, within: &mut Vec<Expr>, pat: HiPat) -> Name {
        match pat.node {
            HiPatNode::Invalid | HiPatNode::Wildcard => {
                let ty = self.lower_type(pat.data);
                let target = self.names.fresh(pat.span, None);
                self.context.add(target, ty);
                target
            }
            HiPatNode::Name(name) => name,
            HiPatNode::Tuple(a, b) => {
                let ty = self.lower_type(pat.data);
                let target = self.names.fresh(pat.span, None);
                self.context.add(target, ty);

                let mut proj = |_: &mut Lowerer<'a>, name, value, ndx, span, ty| {
                    within.push(Expr {
                        node: ExprNode::Proj {
                            name,
                            of: value,
                            at: ndx,
                        },
                        span,
                        ty,
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
        let ty = self.lower_type(ex.data.clone());
        let mut seq = Vec::new();
        let value = self.make_expr(&mut seq, ex);
        let span = value.span;
        ExprSeq::new(
            span,
            ty,
            seq,
            Branch {
                node: BranchNode::Return(value),
                span,
                ty,
            },
        )
    }

    fn make_expr(&mut self, within: &mut Vec<Expr>, ex: HiExpr) -> Value {
        let ty = self.lower_type(ex.data);
        let node = match ex.node {
            HiExprNode::Int(i) => ValueNode::Int(i),
            HiExprNode::Name(name) => ValueNode::Name(name),
            HiExprNode::Lam(param, body) => {
                let mut bodys = Vec::new();

                let param = self.destruct_expr(&mut bodys, param);
                let body = self.lower_expr(*body);

                bodys.extend(body.exprs);

                let body = ExprSeq::new(body.span, body.ty, bodys, body.branch);

                let name = self.names.fresh(ex.span, None);
                self.context.add(name, ty);

                within.push(Expr {
                    node: ExprNode::Function { name, param, body },
                    span: ex.span,
                    ty,
                });

                ValueNode::Name(name)
            }

            HiExprNode::App(fun, arg) => {
                let fun = if let Value {
                    node: ValueNode::Name(name),
                    ..
                } = self.make_expr(within, *fun)
                {
                    name
                } else {
                    unreachable!()
                };

                let arg = self.make_expr(within, *arg);

                let name = self.names.fresh(ex.span, None);
                self.context.add(name, ty);

                within.push(Expr {
                    node: ExprNode::Apply { name, fun, arg },
                    span: ex.span,
                    ty,
                });

                ValueNode::Name(name)
            }

            HiExprNode::Tuple(t, u) => {
                let t = self.make_expr(within, *t);
                let u = self.make_expr(within, *u);

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

                ValueNode::Name(name)
            }

            HiExprNode::Hole => {
                self.messages.at(ex.span).elab_report_hole({
                    let prettier = Prettier::new(self.names, &self.types);
                    prettier.pretty_type(&ty)
                });

                ValueNode::Invalid
            }

            HiExprNode::Invalid => ValueNode::Invalid,

            HiExprNode::Anno(..) => unreachable!(),
        };

        Value {
            node,
            span: ex.span,
            ty,
        }
    }
}
