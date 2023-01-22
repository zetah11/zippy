use super::path::NamePart;
use super::Resolver;
use crate::resolved::{Expr, ExprNode};
use crate::unresolved;

impl Resolver<'_> {
    pub fn resolve_expr(&mut self, expr: unresolved::Expr) -> Expr {
        let node = match expr.node {
            unresolved::ExprNode::Num(v) => ExprNode::Num(v),

            unresolved::ExprNode::Name(name) => match self.lookup(expr.span, name) {
                Some(name) => ExprNode::Name(name),
                None => ExprNode::Invalid,
            },

            unresolved::ExprNode::Lam(id, param, body) => {
                self.in_scope(NamePart::Scope(id), |this| {
                    let param = this.resolve_pat(param);
                    let body = Box::new(this.resolve_expr(*body));

                    ExprNode::Lam(param, body)
                })
            }

            unresolved::ExprNode::App(x, y) => {
                let x = Box::new(self.resolve_expr(*x));
                let y = Box::new(self.resolve_expr(*y));

                ExprNode::App(x, y)
            }

            unresolved::ExprNode::Inst(x, args) => {
                let x = Box::new(self.resolve_expr(*x));
                let args = args.into_iter().map(|ty| self.resolve_type(ty)).collect();

                ExprNode::Inst(x, args)
            }

            unresolved::ExprNode::Tuple(x, y) => {
                let x = Box::new(self.resolve_expr(*x));
                let y = Box::new(self.resolve_expr(*y));

                ExprNode::Tuple(x, y)
            }

            unresolved::ExprNode::Anno(x, ty) => {
                let x = Box::new(self.resolve_expr(*x));
                let ty = self.resolve_type(ty);

                ExprNode::Anno(x, ty)
            }

            unresolved::ExprNode::Hole => ExprNode::Hole,
            unresolved::ExprNode::Invalid => ExprNode::Invalid,
        };

        Expr {
            node,
            span: expr.span,
        }
    }
}
