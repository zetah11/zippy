use zippy_common::hir2::{self, Because, Type};

use super::Typer;
use crate::resolved;

impl Typer<'_> {
    pub fn check(&mut self, because: Because, expr: &resolved::Expr, against: Type) -> hir2::Expr {
        let (node, ty) = match &expr.node {
            resolved::ExprNode::Num(v) => {
                let ty = self.type_number(because, expr.span, against);
                (hir2::ExprNode::Num(v.clone()), ty)
            }

            resolved::ExprNode::Lam(pat, body) => {
                // todo
                let (t, u) = self.type_function(because.clone(), expr.span, against);
                let pat = self.bind_pat(pat, t.clone());
                let body = Box::new(self.check(because, body, u.clone()));

                let ty = Type::Fun(Box::new(t), Box::new(u));
                (hir2::ExprNode::Lam(pat, body), ty)
            }

            resolved::ExprNode::Tuple(x, y) => {
                let (t, u) = self.type_tuple(expr.span, against);
                let x = Box::new(self.check(because.clone(), x, t.clone()));
                let y = Box::new(self.check(because, y, u.clone()));

                let ty = Type::Product(Box::new(t), Box::new(u));
                (hir2::ExprNode::Tuple(x, y), ty)
            }

            resolved::ExprNode::Hole => (hir2::ExprNode::Hole, against),

            _ => {
                let expr = Box::new(self.infer(expr));
                let coerce = self.assign(expr.span, &against, &expr.data);
                (hir2::ExprNode::Coerce(expr, coerce), against)
            }
        };

        hir2::Expr {
            node,
            span: expr.span,
            data: ty,
        }
    }
}
