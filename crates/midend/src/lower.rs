use std::collections::{HashMap, HashSet};

use log::{debug, trace};

use crate::Driver;
use common::message::{Messages, Span};
use common::mir::pretty::Prettier;
use common::mir::{
    Branch, BranchNode, Context, Decls, Expr, ExprNode, ExprSeq, Type, TypeId, Types, Value,
    ValueDef, ValueNode,
};
use common::names::{Name, Names};
use common::thir::{self, merge_insts, UniVar};

type HiType = thir::Type;
type HiPat = thir::Pat<HiType>;
type HiPatNode = thir::PatNode<HiType>;
type HiExpr = thir::Expr<HiType>;
type HiExprNode = thir::ExprNode<HiType>;
type HiValueDef = thir::ValueDef<HiType>;
type HiDecls = thir::Decls<HiType>;
type HiCtx = thir::Context;

pub fn lower(
    driver: &mut impl Driver,
    subst: &HashMap<UniVar, (HashMap<Name, HiType>, HiType)>,
    names: &mut Names,
    context: HiCtx,
    decls: HiDecls,
) -> (Types, Context, Decls) {
    debug!("beginning lowering");
    let mut lowerer = Lowerer::new(names, subst);

    lowerer.lower_context(context);
    let context = lowerer.names.root();
    let decls = lowerer.lower_decls(context, decls);

    driver.report(lowerer.messages);

    trace!("done lowering");

    (lowerer.types, lowerer.context, decls)
}

#[derive(Debug)]
struct Lowerer<'a> {
    types: Types,
    names: &'a mut Names,
    subst: &'a HashMap<UniVar, (HashMap<Name, HiType>, HiType)>,
    messages: Messages,
    context: Context,
    polymorphic: HashSet<Name>,
    templates: HashMap<Name, HiValueDef>,

    values: Vec<ValueDef>,
}

impl<'a> Lowerer<'a> {
    fn new(
        names: &'a mut Names,
        subst: &'a HashMap<UniVar, (HashMap<Name, HiType>, HiType)>,
    ) -> Self {
        Self {
            types: Types::new(),
            names,
            subst,
            messages: Messages::new(),
            context: Context::new(),
            polymorphic: HashSet::new(),
            templates: HashMap::new(),

            values: Vec::new(),
        }
    }

    fn lower_context(&mut self, context: HiCtx) {
        for name in context.polymorphic_names() {
            self.polymorphic.insert(name);
        }

        for (name, ty) in context {
            let span = self.names.get_span(&name);
            let Some(ty) = self.lower_type(span, &HashMap::new(), ty) else { continue; };
            self.context.add(name, ty);
        }
    }

    fn lower_decls(&mut self, ctx: Name, decls: HiDecls) -> Decls {
        let mut monomorphic = Vec::new();

        for def in decls.values {
            if def.implicits.is_empty() {
                monomorphic.push(def);
            } else {
                match def.pat.node {
                    HiPatNode::Name(name) => {
                        self.templates.insert(name, def);
                    }

                    _ => todo!("polymorphic destruction"),
                }
            }
        }

        for def in monomorphic {
            self.destruct_binding(&HashMap::new(), ctx, def.span, def.pat, def.bind);
        }

        Decls::new(self.values.drain(..).collect())
    }

    fn lower_type(&mut self, at: Span, inst: &HashMap<Name, HiType>, ty: HiType) -> Option<TypeId> {
        Some(match ty {
            HiType::Name(name) => {
                let Some(ty) = inst.get(&name) else {
                    return None;
                };

                self.lower_type(at, inst, ty.clone())?
            }

            HiType::Instantiated(ty, other_inst) => {
                let inst = merge_insts(inst, &other_inst);
                self.lower_type(at, &inst, *ty)?
            }

            HiType::Fun(t, u) => {
                let t = self.lower_type(at, inst, *t)?;
                let u = self.lower_type(at, inst, *u)?;

                self.types.add(Type::Fun(vec![t], vec![u]))
            }

            HiType::Product(t, u) => {
                let t = self.lower_type(at, inst, *t)?;
                let u = self.lower_type(at, inst, *u)?;

                self.types.add(Type::Product(vec![t, u]))
            }

            HiType::Range(lo, hi) => self.types.add(Type::Range(lo, hi)),

            HiType::Var(_, v) => {
                if let Some((other_inst, ty)) = self.subst.get(&v).cloned() {
                    let inst = merge_insts(inst, &other_inst);
                    self.lower_type(at, &inst, ty)?
                } else {
                    self.messages.at(at).tyck_ambiguous();
                    self.types.add(Type::Invalid)
                }
            }

            HiType::Invalid => self.types.add(Type::Invalid),
            HiType::Number => unreachable!(),
        })
    }

    fn destruct_binding(
        &mut self,
        inst: &HashMap<Name, HiType>,
        ctx: Name,
        span: Span,
        pat: HiPat,
        bind: HiExpr,
    ) {
        let bind = self.lower_expr(inst, ctx, bind);
        match pat.node {
            HiPatNode::Invalid | HiPatNode::Wildcard => {}
            HiPatNode::Name(name) => self.values.push(ValueDef { span, name, bind }),

            HiPatNode::Tuple(a, b) => {
                let ty = self.lower_type(pat.span, inst, pat.data).unwrap();

                let name = self.names.fresh(pat.span, ctx);
                self.context.add(name, ty);

                self.values.push(ValueDef { name, span, bind });

                let mut within = Vec::new();

                let mut proj = |lowerer: &mut Lowerer<'a>, name, of, at, span, ty| {
                    let inner = lowerer.names.fresh(span, ctx);
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
                                node: BranchNode::Return(vec![Value {
                                    node: ValueNode::Name(inner),
                                    span,
                                    ty,
                                }]),
                                span,
                                ty,
                            },
                        },
                    })
                };

                println!("{:?}", self.names.get_path(&name).1);
                self.make_proj(inst, ctx, *a, name, 0, &mut proj);
                self.make_proj(inst, ctx, *b, name, 1, &mut proj);

                self.values.extend(within);
            }

            HiPatNode::Anno(..) => unreachable!(),
        }
    }

    fn destruct_expr(
        &mut self,
        inst: &HashMap<Name, HiType>,
        within: &mut Vec<Expr>,
        ctx: Name,
        pat: HiPat,
    ) -> Name {
        match pat.node {
            HiPatNode::Invalid | HiPatNode::Wildcard => {
                let ty = self.lower_type(pat.span, inst, pat.data).unwrap();
                let target = self.names.fresh(pat.span, ctx);
                self.context.add(target, ty);
                target
            }
            HiPatNode::Name(name) => name,
            HiPatNode::Tuple(a, b) => {
                let ty = self.lower_type(pat.span, inst, pat.data).unwrap();
                let target = self.names.fresh(pat.span, ctx);
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

                self.make_proj(inst, ctx, *a, target, 0, &mut proj);
                self.make_proj(inst, ctx, *b, target, 1, &mut proj);

                target
            }

            HiPatNode::Anno(..) => unreachable!(),
        }
    }

    fn make_proj<F>(
        &mut self,
        inst: &HashMap<Name, HiType>,
        ctx: Name,
        pat: HiPat,
        value: Name,
        ndx: usize,
        proj: &mut F,
    ) where
        F: FnMut(&mut Lowerer<'a>, Name, Name, usize, Span, TypeId),
    {
        match pat.node {
            HiPatNode::Invalid | HiPatNode::Wildcard => {}
            HiPatNode::Name(name) => {
                let ty = self.lower_type(pat.span, inst, pat.data).unwrap();
                proj(self, name, value, ndx, pat.span, ty);
            }

            HiPatNode::Tuple(a, b) => {
                let ty = self.lower_type(pat.span, inst, pat.data).unwrap();
                let projd = self.names.fresh(pat.span, ctx);
                self.context.add(projd, ty);

                proj(self, projd, value, ndx, pat.span, ty);

                self.make_proj(inst, ctx, *a, projd, 0, proj);
                self.make_proj(inst, ctx, *b, projd, 1, proj);
            }

            HiPatNode::Anno(..) => unreachable!(),
        }
    }

    fn lower_expr(&mut self, inst: &HashMap<Name, HiType>, ctx: Name, ex: HiExpr) -> ExprSeq {
        let ty = self.lower_type(ex.span, inst, ex.data.clone()).unwrap();
        let mut seq = Vec::new();
        let value = self.make_expr(inst, &mut seq, ctx, ex);
        let span = value.span;
        ExprSeq::new(
            span,
            ty,
            seq,
            Branch {
                node: BranchNode::Return(vec![value]),
                span,
                ty,
            },
        )
    }

    fn make_expr(
        &mut self,
        inst: &HashMap<Name, HiType>,
        within: &mut Vec<Expr>,
        ctx: Name,
        ex: HiExpr,
    ) -> Value {
        let ty = self.lower_type(ex.span, inst, ex.data).unwrap();
        let node = match ex.node {
            HiExprNode::Int(i) => ValueNode::Int(i),
            HiExprNode::Name(name) => ValueNode::Name(name),
            HiExprNode::Lam(param, body) => {
                let mut bodys = Vec::new();

                let param = self.destruct_expr(inst, &mut bodys, ctx, param);
                let body = self.lower_expr(inst, ctx, *body);

                bodys.extend(body.exprs);

                let body = ExprSeq::new(body.span, body.ty, bodys, body.branch);

                let name = self.names.fresh(ex.span, ctx);
                self.context.add(name, ty);

                within.push(Expr {
                    node: ExprNode::Function {
                        name,
                        params: vec![param],
                        body,
                    },
                    span: ex.span,
                    ty,
                });

                ValueNode::Name(name)
            }

            HiExprNode::App(fun, arg) => {
                let fun = if let Value {
                    node: ValueNode::Name(name),
                    ..
                } = self.make_expr(inst, within, ctx, *fun)
                {
                    name
                } else {
                    unreachable!()
                };

                let arg = self.make_expr(inst, within, ctx, *arg);

                let name = self.names.fresh(ex.span, ctx);
                self.context.add(name, ty);

                within.push(Expr {
                    node: ExprNode::Apply {
                        names: vec![name],
                        fun,
                        args: vec![arg],
                    },
                    span: ex.span,
                    ty,
                });

                ValueNode::Name(name)
            }

            HiExprNode::Inst(node, args) => {
                let HiExprNode::Name(name) = node.node else { unreachable!()};
                let name = self.instantiate(ex.span, inst, &name, args);
                ValueNode::Name(name)
            }

            HiExprNode::Tuple(t, u) => {
                let t = self.make_expr(inst, within, ctx, *t);
                let u = self.make_expr(inst, within, ctx, *u);

                let name = self.names.fresh(ex.span, ctx);
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

    fn instantiate(
        &mut self,
        at: Span,
        inst: &HashMap<Name, HiType>,
        name: &Name,
        args: Vec<(Span, HiType)>,
    ) -> Name {
        let template = self.templates.get(name).unwrap();
        assert!(template.implicits.len() == args.len());

        let mut inst = inst.clone();
        for ((name, _), (_, ty)) in template.implicits.iter().zip(args) {
            inst.insert(*name, ty);
        }

        let target = self.names.fresh(at, *name);
        let span = template.span;
        let anno = template.anno.clone();
        let bind = template.bind.clone();
        let ty = self.lower_type(at, &inst, anno).unwrap();
        let bind = self.lower_expr(&inst, target, bind);

        self.context.add(target, ty);

        self.values.push(ValueDef {
            span,
            name: target,
            bind,
        });

        target
    }
}
