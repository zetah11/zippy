use std::collections::HashMap;

use zippy_common::names::Names;
use zippy_common::thir::Context;
use zippy_common::{hir, thir};

use super::Name;

#[derive(Debug)]
pub struct Lowerer<'a> {
    names: &'a mut Names,
    context: Context,

    lifted: HashMap<hir::Expr<Name>, Name>,
}

impl<'a> Lowerer<'a> {
    pub fn new(names: &'a mut Names) -> Self {
        Self {
            names,
            context: Context::new(),

            lifted: HashMap::new(),
        }
    }

    pub fn lower(mut self, ex: hir::Decls<Name>) -> (thir::Decls, Context) {
        let res = self.lower_decls(ex);
        (res, self.context)
    }

    fn lower_decls(&mut self, decls: hir::Decls<Name>) -> thir::Decls {
        let mut values = Vec::with_capacity(decls.values.len());
        let mut types = Vec::with_capacity(decls.types.len());

        for def in decls.values {
            let def = self.lower_value_def(&mut values, def);
            values.push(def);
        }

        for def in decls.types {
            let def = self.lower_type_def(&mut values, def);
            types.push(def);
        }

        thir::Decls { values, types }
    }

    fn lower_value_def(
        &mut self,
        values: &mut Vec<thir::ValueDef>,
        def: hir::ValueDef<Name>,
    ) -> thir::ValueDef {
        let pat = self.lower_pat(values, def.pat);
        let _ = def.id;
        let anno = self.lower_type(values, def.anno);
        let bind = self.lower_expr(values, def.bind);

        thir::ValueDef {
            span: def.span,
            implicits: def.implicits,
            pat,
            anno,
            bind,
        }
    }

    fn lower_type_def(
        &mut self,
        values: &mut Vec<thir::ValueDef>,
        def: hir::TypeDef<Name>,
    ) -> thir::TypeDef {
        let pat = self.lower_pat(values, def.pat);
        let _ = def.id;
        let anno = self.lower_type(values, def.anno);
        let bind = self.lower_type(values, def.bind);

        thir::TypeDef {
            span: def.span,
            pat,
            anno,
            bind,
        }
    }

    fn lower_expr(&mut self, values: &mut Vec<thir::ValueDef>, ex: hir::Expr<Name>) -> thir::Expr {
        let node = match ex.node {
            hir::ExprNode::Name(name) => thir::ExprNode::Name(name),
            hir::ExprNode::Num(v) => thir::ExprNode::Num(v),
            hir::ExprNode::Lam(_, param, body) => {
                let param = self.lower_pat(values, param);
                let body = Box::new(self.lower_expr(values, *body));
                thir::ExprNode::Lam(param, body)
            }
            hir::ExprNode::App(fun, arg) => {
                let fun = Box::new(self.lower_expr(values, *fun));
                let arg = Box::new(self.lower_expr(values, *arg));
                thir::ExprNode::App(fun, arg)
            }
            hir::ExprNode::Inst(fun, args) => {
                let fun = Box::new(self.lower_expr(values, *fun));
                let args = args
                    .into_iter()
                    .map(|ty| (ty.span, self.lower_type(values, ty)))
                    .collect();
                thir::ExprNode::Inst(fun, args)
            }
            hir::ExprNode::Anno(ex, ty) => {
                let ex = Box::new(self.lower_expr(values, *ex));
                let anno_span = ty.span;
                let ty = self.lower_type(values, ty);
                thir::ExprNode::Anno(ex, anno_span, ty)
            }
            hir::ExprNode::Tuple(x, y) => {
                let x = Box::new(self.lower_expr(values, *x));
                let y = Box::new(self.lower_expr(values, *y));
                thir::ExprNode::Tuple(x, y)
            }
            hir::ExprNode::Hole => thir::ExprNode::Hole,
            hir::ExprNode::Invalid => thir::ExprNode::Invalid,
        };

        thir::Expr {
            node,
            span: ex.span,
            data: (),
        }
    }

    fn lower_pat(&mut self, values: &mut Vec<thir::ValueDef>, pat: hir::Pat<Name>) -> thir::Pat {
        let node = match pat.node {
            hir::PatNode::Name(name) => thir::PatNode::Name(name),
            hir::PatNode::Tuple(x, y) => {
                let x = Box::new(self.lower_pat(values, *x));
                let y = Box::new(self.lower_pat(values, *y));
                thir::PatNode::Tuple(x, y)
            }
            hir::PatNode::Anno(pat, ty) => {
                let pat = Box::new(self.lower_pat(values, *pat));
                let ty = self.lower_type(values, ty);
                thir::PatNode::Anno(pat, ty)
            }
            hir::PatNode::Wildcard => thir::PatNode::Wildcard,
            hir::PatNode::Invalid => thir::PatNode::Invalid,
        };

        thir::Pat {
            node,
            span: pat.span,
            data: (),
        }
    }

    fn lower_type(&mut self, values: &mut Vec<thir::ValueDef>, ty: hir::Type<Name>) -> thir::Type {
        match ty.node {
            hir::TypeNode::Name(name) => thir::Type::Name(name),
            hir::TypeNode::Range(lo, hi) => {
                let lo = self.lift_expr(values, *lo);
                let hi = self.lift_expr(values, *hi);

                thir::Type::Range(lo, hi)
            }
            hir::TypeNode::Fun(t, u) => {
                let t = Box::new(self.lower_type(values, *t));
                let u = Box::new(self.lower_type(values, *u));
                thir::Type::Fun(t, u)
            }
            hir::TypeNode::Prod(t, u) => {
                let t = Box::new(self.lower_type(values, *t));
                let u = Box::new(self.lower_type(values, *u));
                thir::Type::Product(t, u)
            }
            hir::TypeNode::Type => thir::Type::Type,
            hir::TypeNode::Wildcard => thir::Type::mutable(self.context.fresh()),
            hir::TypeNode::Invalid => thir::Type::Invalid,
        }
    }

    fn lift_expr(&mut self, values: &mut Vec<thir::ValueDef>, ex: hir::Expr<Name>) -> Name {
        if let Some(name) = self.lifted.get(&ex) {
            *name
        } else {
            let root = self.names.root();
            let name = self.names.fresh(ex.span, root);
            self.lifted.insert(ex.clone(), name);

            let ex = self.lower_expr(values, ex);
            values.push(self.make_def(name, ex));

            name
        }
    }

    fn make_def(&mut self, name: Name, ex: thir::Expr) -> thir::ValueDef {
        let span = ex.span;

        thir::ValueDef {
            pat: thir::Pat {
                node: thir::PatNode::Name(name),
                span,
                data: (),
            },
            implicits: vec![],
            anno: thir::Type::Number,
            bind: ex,
            span,
        }
    }
}
