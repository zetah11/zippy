use common::{hir, thir};

use super::{Name, Typer};

impl Typer {
    pub fn lower(&mut self, ex: hir::Decls<Name>) -> thir::Decls {
        self.lower_decls(ex)
    }

    fn lower_decls(&mut self, decls: hir::Decls<Name>) -> thir::Decls {
        let mut values = Vec::with_capacity(decls.values.len());

        for def in decls.values {
            values.push(self.lower_value_def(def));
        }

        thir::Decls { values }
    }

    fn lower_value_def(&mut self, def: hir::ValueDef<Name>) -> thir::ValueDef {
        let pat = self.lower_pat(def.pat);
        let _ = def.id;
        let anno = self.lower_type(def.anno);
        let bind = self.lower_expr(def.bind);

        thir::ValueDef {
            span: def.span,
            pat,
            anno,
            bind,
        }
    }

    fn lower_expr(&mut self, ex: hir::Expr<Name>) -> thir::Expr {
        let node = match ex.node {
            hir::ExprNode::Name(name) => thir::ExprNode::Name(name),
            hir::ExprNode::Int(v) => thir::ExprNode::Int(v),
            hir::ExprNode::Lam(_, param, body) => {
                let param = self.lower_pat(param);
                let body = Box::new(self.lower_expr(*body));
                thir::ExprNode::Lam(param, body)
            }
            hir::ExprNode::App(fun, arg) => {
                let fun = Box::new(self.lower_expr(*fun));
                let arg = Box::new(self.lower_expr(*arg));
                thir::ExprNode::App(fun, arg)
            }
            hir::ExprNode::Anno(ex, ty) => {
                let ex = Box::new(self.lower_expr(*ex));
                let anno_span = ty.span;
                let ty = self.lower_type(ty);
                thir::ExprNode::Anno(ex, anno_span, ty)
            }
            hir::ExprNode::Tuple(x, y) => {
                let x = Box::new(self.lower_expr(*x));
                let y = Box::new(self.lower_expr(*y));
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

    fn lower_pat(&mut self, pat: hir::Pat<Name>) -> thir::Pat {
        let node = match pat.node {
            hir::PatNode::Name(name) => thir::PatNode::Name(name),
            hir::PatNode::Tuple(x, y) => {
                let x = Box::new(self.lower_pat(*x));
                let y = Box::new(self.lower_pat(*y));
                thir::PatNode::Tuple(x, y)
            }
            hir::PatNode::Anno(pat, ty) => {
                let pat = Box::new(self.lower_pat(*pat));
                let ty = self.lower_type(ty);
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

    fn lower_type(&mut self, ty: hir::Type) -> thir::Type {
        match ty.node {
            hir::TypeNode::Range(lo, hi) => thir::Type::Range(lo, hi),
            hir::TypeNode::Fun(t, u) => {
                let t = Box::new(self.lower_type(*t));
                let u = Box::new(self.lower_type(*u));
                thir::Type::Fun(t, u)
            }
            hir::TypeNode::Prod(t, u) => {
                let t = Box::new(self.lower_type(*t));
                let u = Box::new(self.lower_type(*u));
                thir::Type::Product(t, u)
            }
            hir::TypeNode::Wildcard => thir::Type::Var(self.context.fresh()),
            hir::TypeNode::Invalid => thir::Type::Invalid,
        }
    }
}
