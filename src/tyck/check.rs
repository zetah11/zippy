use super::{Expr, ExprNode, Type, Typer};

impl Typer {
    /// Check that an expression conforms to a given type.
    pub fn check(&mut self, ex: Expr, ty: Type) -> Expr<Type> {
        let (node, ty) = match ex.node {
            ExprNode::Int(v) => (ExprNode::Int(v), self.contains_int((v, ex.span), ty)),
            ExprNode::Lam(param, body) => {
                let (t, u) = self.fun_type(ex.span, ty);
                let param = self.bind_pat(param, t.clone());
                let body = self.check(*body, u.clone());
                (
                    ExprNode::Lam(param, Box::new(body)),
                    Type::Fun(Box::new(t), Box::new(u)),
                )
            }

            ExprNode::Hole => (ExprNode::Hole, self.report_hole(ex.span, ty)),

            _ => {
                let ex = self.infer(ex);
                self.assignable(ex.span, ty, ex.data.clone());
                return ex;
            }
        };

        Expr {
            node,
            span: ex.span,
            data: ty,
        }
    }
}
