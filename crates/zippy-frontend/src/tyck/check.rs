use log::trace;
use zippy_common::thir::Because;

use super::{Expr, ExprNode, Type, Typer};

impl Typer<'_> {
    /// Check that an expression conforms to a given type.
    pub fn check(&mut self, because: Because, ex: Expr, ty: Type) -> Expr<Type> {
        let pretty = self.pretty(&ty);
        let (node, ty) = match ex.node {
            ExprNode::Int(v) => {
                trace!("checking int against {}", pretty);
                (ExprNode::Int(v), self.int_type(ex.span, because, ty))
            }

            ExprNode::Lam(param, body) => {
                trace!("checking lambda against {}", pretty);
                let (t, u) = self.fun_type(ex.span, ty);
                let param = self.bind_pat(param, t.clone());
                let body = self.check(because, *body, u.clone());
                (
                    ExprNode::Lam(param, Box::new(body)),
                    Type::Fun(Box::new(t), Box::new(u)),
                )
            }

            ExprNode::Tuple(x, y) => {
                trace!("checking tuple against {}", pretty);
                let (t, u) = self.tuple_type(ex.span, ty);
                let x = Box::new(self.check(because.clone(), *x, t.clone()));
                let y = Box::new(self.check(because, *y, u.clone()));

                (
                    ExprNode::Tuple(x, y),
                    Type::Product(Box::new(t), Box::new(u)),
                )
            }

            ExprNode::Hole => {
                trace!("checking hole against {}", pretty);
                (ExprNode::Hole, self.hole_type(ex.span, ty))
            }

            _ => {
                trace!("subsumption");
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
