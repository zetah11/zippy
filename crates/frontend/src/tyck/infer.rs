use common::thir::Because;

use super::{Expr, ExprNode, Type, Typer};

impl Typer {
    /// Infer the type of an expression.
    pub fn infer(&mut self, ex: Expr) -> Expr<Type> {
        let (node, ty) = match ex.node {
            ExprNode::Name(name) => {
                let ty = self.context.get(&name).clone();
                (ExprNode::Name(name), ty)
            }

            ExprNode::Int(i) => {
                let var = self.context.fresh();
                self.int_type(ex.span, Because::Unified(ex.span), Type::Var(var));
                (ExprNode::Int(i), Type::Var(var))
            }

            ExprNode::Tuple(a, b) => {
                let t = self.context.fresh();
                let u = self.context.fresh();
                let a = Box::new(self.check(Because::Inferred(ex.span, None), *a, Type::Var(t)));
                let b = Box::new(self.check(Because::Inferred(ex.span, None), *b, Type::Var(u)));
                (
                    ExprNode::Tuple(a, b),
                    Type::Product(Box::new(Type::Var(t)), Box::new(Type::Var(u))),
                )
            }

            ExprNode::Lam(param, body) => {
                let param = self.bind_fresh(param);
                let body = Box::new(self.infer(*body));

                let t = param.data.clone();
                let u = body.data.clone();

                let ty = Type::Fun(Box::new(t), Box::new(u));
                (ExprNode::Lam(param, body), ty)
            }

            ExprNode::App(fun, arg) => {
                let fun = self.infer(*fun);
                let (t, u) = self.fun_type(ex.span, fun.data.clone());
                let arg = self.check(Because::Inferred(fun.span, None), *arg, t);
                (ExprNode::App(Box::new(fun), Box::new(arg)), u)
            }

            ExprNode::Anno(ex, anno_span, ty) => {
                return self.check(Because::Annotation(anno_span), *ex, ty);
            }

            ExprNode::Hole => (ExprNode::Hole, Type::Var(self.context.fresh())),

            ExprNode::Invalid => (ExprNode::Invalid, Type::Invalid),
            /*_ => {
                self.messages.at(ex.span).tyck_ambiguous();
                (ExprNode::Invalid, Type::Invalid)
            }*/
        };

        Expr {
            node,
            span: ex.span,
            data: ty,
        }
    }
}
