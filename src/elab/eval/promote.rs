//! Promotion converts an [`Irreducible`](super::Irreducible) back to an `ExprSeq`.

use super::{Irreducible, IrreducibleNode, Lowerer};
use crate::mir::{Expr, ExprNode, ExprSeq};
use crate::mir::{Value, ValueNode};
use crate::Driver;

impl<D: Driver> Lowerer<'_, D> {
    pub fn promote(&mut self, value: Irreducible) -> ExprSeq {
        if let IrreducibleNode::Quote(exprs) = value.node {
            exprs
        } else {
            let mut exprs = ExprSeq::new(value.span, value.ty);
            let Irreducible { span, ty, .. } = value;
            let res = self.make_irr(&mut exprs, value);
            exprs.push(Expr {
                node: ExprNode::Produce(res),
                span,
                ty,
            });

            exprs
        }
    }

    fn make_irr(&mut self, within: &mut ExprSeq, value: Irreducible) -> Value {
        let node = match value.node {
            IrreducibleNode::Invalid => ValueNode::Invalid,
            IrreducibleNode::Integer(i) => {
                self.check_int_range(value.span, i, &value.ty);
                ValueNode::Int(i)
            }
            IrreducibleNode::Lambda(param, body, _) => {
                let name = self.names.fresh(value.span, None);
                self.context.add(name, value.ty);

                let body = self.promote(*body);

                within.push(Expr {
                    node: ExprNode::Function { name, param, body },
                    span: value.span,
                    ty: value.ty,
                });

                ValueNode::Name(name)
            }

            IrreducibleNode::Tuple(values) => {
                let values = values
                    .into_iter()
                    .map(|value| self.make_irr(within, value))
                    .collect();

                let name = self.names.fresh(value.span, None);
                self.context.add(name, value.ty);

                within.push(Expr {
                    node: ExprNode::Tuple { name, values },
                    span: value.span,
                    ty: value.ty,
                });

                ValueNode::Name(name)
            }

            IrreducibleNode::Quote(exprs) => {
                let last = exprs.exprs.last().unwrap();
                if let ExprNode::Produce(ref value) = last.node {
                    let value = value.clone();
                    within.extend(exprs.exprs);
                    return value;
                } else {
                    unreachable!()
                }
            }
        };

        Value {
            node,
            span: value.span,
            ty: value.ty,
        }
    }
}
