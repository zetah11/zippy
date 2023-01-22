use super::path::NamePart;
use super::Resolver;
use crate::unresolved::{Expr, ExprNode};

impl Resolver<'_> {
    pub fn declare_expr(&mut self, expr: &Expr) {
        match &expr.node {
            ExprNode::Name(_) | ExprNode::Num(_) | ExprNode::Hole | ExprNode::Invalid => {}
            ExprNode::Lam(id, param, body) => {
                self.in_scope_mut(expr.span, NamePart::Scope(*id), |this| {
                    this.declare_pat(param);
                    this.declare_expr(body);
                });
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
