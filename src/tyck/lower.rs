use super::{tree, Typer};
use crate::hir;
use crate::resolve::names::Name;

impl Typer {
    pub fn lower(&mut self, ex: hir::Decls<Name>) -> tree::Decls {
        self.lower_decls(ex)
    }

    fn lower_decls(&mut self, decls: hir::Decls<Name>) -> tree::Decls {
        let mut values = Vec::with_capacity(decls.values.len());

        for def in decls.values {
            values.push(self.lower_value_def(def));
        }

        tree::Decls { values }
    }

    fn lower_value_def(&mut self, def: hir::ValueDef<Name>) -> tree::ValueDef {
        let pat = Self::lower_pat(def.pat);
        let _ = def.id;
        let anno = self.lower_type(def.anno);
        let bind = self.lower_expr(def.bind);

        tree::ValueDef {
            span: def.span,
            pat,
            anno,
            bind,
        }
    }

    fn lower_expr(&mut self, ex: hir::Expr<Name>) -> tree::Expr {
        let node = match ex.node {
            hir::ExprNode::Name(name) => tree::ExprNode::Name(name),
            hir::ExprNode::Int(v) => tree::ExprNode::Int(v),
            hir::ExprNode::Lam(_, param, body) => {
                let param = Self::lower_pat(param);
                let body = Box::new(self.lower_expr(*body));
                tree::ExprNode::Lam(param, body)
            }
            hir::ExprNode::App(fun, arg) => {
                let fun = Box::new(self.lower_expr(*fun));
                let arg = Box::new(self.lower_expr(*arg));
                tree::ExprNode::App(fun, arg)
            }
            hir::ExprNode::Anno(ex, ty) => {
                let ex = Box::new(self.lower_expr(*ex));
                let anno_span = ty.span;
                let ty = self.lower_type(ty);
                tree::ExprNode::Anno(ex, anno_span, ty)
            }
            hir::ExprNode::Tuple(x, y) => {
                let x = Box::new(self.lower_expr(*x));
                let y = Box::new(self.lower_expr(*y));
                tree::ExprNode::Tuple(x, y)
            }
            hir::ExprNode::Hole => tree::ExprNode::Hole,
            hir::ExprNode::Invalid => tree::ExprNode::Invalid,
        };

        tree::Expr {
            node,
            span: ex.span,
            data: (),
        }
    }

    fn lower_pat(pat: hir::Pat<Name>) -> tree::Pat {
        let node = match pat.node {
            hir::PatNode::Name(name) => tree::PatNode::Name(name),
            hir::PatNode::Tuple(x, y) => {
                let x = Box::new(Self::lower_pat(*x));
                let y = Box::new(Self::lower_pat(*y));
                tree::PatNode::Tuple(x, y)
            }
            hir::PatNode::Wildcard => tree::PatNode::Wildcard,
            hir::PatNode::Invalid => tree::PatNode::Invalid,
        };

        tree::Pat {
            node,
            span: pat.span,
            data: (),
        }
    }

    fn lower_type(&mut self, ty: hir::Type) -> tree::Type {
        match ty.node {
            hir::TypeNode::Range(lo, hi) => tree::Type::Range(lo, hi),
            hir::TypeNode::Fun(t, u) => {
                let t = Box::new(self.lower_type(*t));
                let u = Box::new(self.lower_type(*u));
                tree::Type::Fun(t, u)
            }
            hir::TypeNode::Prod(t, u) => {
                let t = Box::new(self.lower_type(*t));
                let u = Box::new(self.lower_type(*u));
                tree::Type::Product(t, u)
            }
            hir::TypeNode::Wildcard => tree::Type::Var(self.context.fresh()),
            hir::TypeNode::Invalid => tree::Type::Invalid,
        }
    }
}
