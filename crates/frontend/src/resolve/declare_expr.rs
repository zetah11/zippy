use common::hir::{Expr, ExprNode};

use super::Resolver;

impl Resolver {
    pub fn declare_expr(&mut self, expr: &Expr) {
        match &expr.node {
            ExprNode::Name(_) | ExprNode::Int(_) | ExprNode::Hole | ExprNode::Invalid => {}
            ExprNode::Lam(id, param, body) => {
                self.enter(param.span, *id);
                self.declare_pat(param);
                self.declare_expr(body);
                self.exit();
            }

            ExprNode::App(fun, arg) => {
                self.declare_expr(fun);
                self.declare_expr(arg);
            }

            ExprNode::Inst(fun, _args) => {
                self.declare_expr(fun);
            }

            ExprNode::Tuple(x, y) => {
                self.declare_expr(x);
                self.declare_expr(y);
            }

            ExprNode::Anno(expr, _) => self.declare_expr(expr),
        }
    }
}
