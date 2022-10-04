use super::tree;
use crate::hir;
use crate::resolve::names::Name;

pub fn lower(ex: hir::Decls<Name>) -> tree::Decls {
    lower_decls(ex)
}

fn lower_decls(decls: hir::Decls<Name>) -> tree::Decls {
    let mut values = Vec::with_capacity(decls.values.len());

    for def in decls.values {
        values.push(lower_value_def(def));
    }

    tree::Decls { values }
}

fn lower_value_def(def: hir::ValueDef<Name>) -> tree::ValueDef {
    let pat = lower_pat(def.pat);
    let _ = def.id;
    let anno = def.anno.map(lower_type).unwrap();
    let bind = lower_expr(def.bind);

    tree::ValueDef {
        span: def.span,
        pat,
        anno,
        bind,
    }
}

fn lower_expr(ex: hir::Expr<Name>) -> tree::Expr {
    let node = match ex.node {
        hir::ExprNode::Name(name) => tree::ExprNode::Name(name),
        hir::ExprNode::Int(v) => tree::ExprNode::Int(v),
        hir::ExprNode::Lam(_, param, body) => {
            let param = lower_pat(param);
            let body = Box::new(lower_expr(*body));
            tree::ExprNode::Lam(param, body)
        }
        hir::ExprNode::App(fun, arg) => {
            let fun = Box::new(lower_expr(*fun));
            let arg = Box::new(lower_expr(*arg));
            tree::ExprNode::App(fun, arg)
        }
        hir::ExprNode::Anno(ex, ty) => {
            let ex = Box::new(lower_expr(*ex));
            let ty = lower_type(ty);
            tree::ExprNode::Anno(ex, ty)
        }
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
        hir::PatNode::Invalid => tree::PatNode::Invalid,
    };

    tree::Pat {
        node,
        span: pat.span,
        data: (),
    }
}

fn lower_type(ty: hir::Type) -> tree::Type {
    match ty.node {
        hir::TypeNode::Range(lo, hi) => tree::Type::Range(lo, hi),
        hir::TypeNode::Fun(t, u) => {
            let t = Box::new(lower_type(*t));
            let u = Box::new(lower_type(*u));
            tree::Type::Fun(t, u)
        }
        hir::TypeNode::Invalid => tree::Type::Invalid,
    }
}
