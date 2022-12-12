use zippy_common::hir::{Expr, ExprNode};
use zippy_common::names::{Actual, Name};

use super::Resolver;

impl Resolver {
    pub fn resolve_expr(&mut self, expr: Expr) -> Expr<Name> {
        let node = match expr.node {
            ExprNode::Hole => ExprNode::Hole,
            ExprNode::Invalid => ExprNode::Invalid,
            ExprNode::Num(v) => ExprNode::Num(v),
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

            ExprNode::Inst(fun, args) => {
                let fun = Box::new(self.resolve_expr(*fun));
                let args = args.into_iter().map(|ty| self.resolve_type(ty)).collect();
                ExprNode::Inst(fun, args)
            }

            ExprNode::Tuple(x, y) => {
                let x = Box::new(self.resolve_expr(*x));
                let y = Box::new(self.resolve_expr(*y));
                ExprNode::Tuple(x, y)
            }

            ExprNode::Anno(expr, ty) => {
                let expr = Box::new(self.resolve_expr(*expr));
                let ty = self.resolve_type(ty);
                ExprNode::Anno(expr, ty)
            }
        };

        Expr {
            node,
            span: expr.span,
        }
    }
}
