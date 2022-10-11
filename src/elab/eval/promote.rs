//! Promotion converts an [`Irreducible`](super::Irreducible) back to an `ExprSeq`.

use super::Irreducible;
use super::Lowerer;
use crate::message::Span;
use crate::mir::Type;
use crate::mir::TypeId;
use crate::mir::Value;
use crate::mir::{Expr, ExprNode, ExprSeq};
use crate::Driver;

impl<D: Driver> Lowerer<'_, D> {
    pub fn promote(&mut self, span: Span, ty: TypeId, value: Irreducible) -> ExprSeq {
        if let Irreducible::Quote(exprs) = value {
            exprs
        } else {
            let mut exprs = ExprSeq::default();
            let res = self.make_irr(&mut exprs, span, ty, value);
            exprs.push(Expr {
                node: ExprNode::Produce(res),
                span,
                ty,
            });

            exprs
        }
    }

    fn make_irr(
        &mut self,
        within: &mut ExprSeq,
        span: Span,
        ty: TypeId,
        value: Irreducible,
    ) -> Value {
        match value {
            Irreducible::Invalid => Value::Invalid,
            Irreducible::Integer(i) => Value::Int(i),
            Irreducible::Lambda(param, body, _) => {
                let name = self.names.fresh(span, None);
                self.context.add(name, ty);

                within.push(Expr {
                    node: ExprNode::Function { name, param, body },
                    span,
                    ty,
                });

                Value::Name(name)
            }

            Irreducible::Tuple(values) => {
                let ties = match self.types.get(&ty) {
                    Type::Product(t, u) => vec![*t, *u],
                    Type::Invalid => vec![ty; values.len()],
                    _ => unreachable!(),
                };

                let values = values
                    .into_iter()
                    .zip(ties)
                    .map(|(value, ty)| self.make_irr(within, span, ty, value))
                    .collect();

                let name = self.names.fresh(span, None);
                self.context.add(name, ty);

                within.push(Expr {
                    node: ExprNode::Tuple { name, values },
                    span,
                    ty,
                });

                Value::Name(name)
            }

            Irreducible::Quote(exprs) => {
                let last = exprs.exprs.last().unwrap();
                if let ExprNode::Produce(ref value) = last.node {
                    let value = value.clone();
                    within.extend(exprs.exprs);
                    value
                } else {
                    unreachable!()
                }
            }
        }
    }
}
