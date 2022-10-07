use super::{Actual, Name, Resolver};
use crate::hir::{Expr, ExprNode};

impl Resolver {
    pub fn resolve_expr(&mut self, expr: Expr) -> Expr<Name> {
        let node = match expr.node {
            ExprNode::Hole => ExprNode::Hole,
            ExprNode::Invalid => ExprNode::Invalid,
            ExprNode::Int(v) => ExprNode::Int(v),
            ExprNode::Name(name) => match self.lookup_name(expr.span, Actual::Lit(name)) {
                Some(name) => ExprNode::Name(name),
                None => ExprNode::Invalid,
            },

            ExprNode::Lam(id, param, body) => {
                self.enter(expr.span, id);
                let param = self.resolve_pat(param);
                let body = Box::new(self.resolve_expr(*body));
                self.exit();
                ExprNode::Lam(id, param, body)
            }

            ExprNode::App(fun, arg) => {
                let fun = Box::new(self.resolve_expr(*fun));
                let arg = Box::new(self.resolve_expr(*arg));
                ExprNode::App(fun, arg)
            }

            ExprNode::Anno(expr, ty) => {
                let expr = Box::new(self.resolve_expr(*expr));
                ExprNode::Anno(expr, ty)
            }
        };

        Expr {
            node,
            span: expr.span,
        }
    }
}
