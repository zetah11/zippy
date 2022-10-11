//! Promotion converts an [`Irreducible`](super::Irreducible) back to an `ExprSeq`.

use super::Irreducible;
use super::Lowerer;
use crate::message::Span;
use crate::mir::TypeId;
use crate::mir::Value;
use crate::mir::{Expr, ExprNode, ExprSeq};
use crate::Driver;

impl<D: Driver> Lowerer<'_, D> {
    pub fn promote(&mut self, span: Span, typ: TypeId, value: Irreducible) -> ExprSeq {
        let exprs = match value {
            Irreducible::Integer(i) => vec![Expr {
                node: ExprNode::Produce(Value::Int(i)),
                span,
                typ,
            }],

            Irreducible::Invalid => vec![Expr {
                node: ExprNode::Produce(Value::Invalid),
                span,
                typ,
            }],

            Irreducible::Lambda(param, body, _) => {
                let name = self.names.fresh(span, None);
                vec![
                    Expr {
                        node: ExprNode::Function { name, param, body },
                        span,
                        typ,
                    },
                    Expr {
                        node: ExprNode::Produce(Value::Name(name)),
                        span,
                        typ,
                    },
                ]
            }

            Irreducible::Tuple(_values) => {
                todo!()
            }

            Irreducible::Quote(exprs) => return exprs,
        };

        ExprSeq { exprs }
    }
}
