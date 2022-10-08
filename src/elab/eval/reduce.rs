use super::Evaluator;
use crate::mir::{Expr, ExprNode};
use crate::Driver;

impl<'a, D: Driver> Evaluator<'a, D> {
    pub fn reduce(&mut self, ex: Expr) -> Expr {
        let node = match ex.node {
            ExprNode::Invalid => ExprNode::Invalid,
            ExprNode::Int(v) => ExprNode::Int(v),
            ExprNode::Name(name) => match self.lookup(&name) {
                Some(value) => {
                    let value = value.clone();
                    return value;
                }
                None => ExprNode::Name(name),
            },
            ExprNode::Lam(param, body) => ExprNode::Lam(param, Box::new(self.reduce(*body))),
            ExprNode::App(fun, arg) => {
                let fun = self.reduce(*fun);
                let arg = self.reduce(*arg);

                match fun.node {
                    ExprNode::Lam(param, body) => {
                        self.enter();
                        self.bind(param, arg);
                        let body = self.reduce(*body);
                        self.exit();
                        return body;
                    }

                    e => ExprNode::App(
                        Box::new(Expr {
                            node: e,
                            span: fun.span,
                            typ: fun.typ,
                        }),
                        Box::new(arg),
                    ),
                }
            }
        };

        Expr {
            node,
            span: ex.span,
            typ: ex.typ,
        }
    }
}
