use super::{Expr, ExprNode, Type, Typer};

impl Typer {
    /// Infer the type of an expression.
    pub fn infer(&mut self, ex: Expr) -> Expr<Type> {
        let (node, ty) = match ex.node {
            ExprNode::Name(name) => {
                let ty = self.context.get(&name).clone();
                (ExprNode::Name(name), ty)
            }

            ExprNode::App(fun, arg) => {
                let fun = self.infer(*fun);
                let (t, u) = self.fun_type(ex.span, fun.data.clone());
                let arg = self.check(*arg, t);
                (ExprNode::App(Box::new(fun), Box::new(arg)), u)
            }

            ExprNode::Anno(ex, ty) => {
                return self.check(*ex, ty);
            }

            ExprNode::Invalid => (ExprNode::Invalid, Type::Invalid),

            _ => {
                self.messages.at(ex.span).tyck_ambiguous();
                (ExprNode::Invalid, Type::Invalid)
            }
        };

        Expr {
            node,
            span: ex.span,
            data: ty,
        }
    }
}
