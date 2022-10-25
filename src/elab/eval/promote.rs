//! Promotion converts an [`Irreducible`](super::Irreducible) back to an `ExprSeq`.

use super::{Irreducible, IrreducibleNode, Lowerer};
use crate::mir::{Branch, BranchNode, Expr, ExprNode, ExprSeq};
use crate::mir::{Value, ValueNode};
use crate::Driver;

impl<D: Driver> Lowerer<'_, D> {
    pub fn promote(&mut self, value: Irreducible) -> ExprSeq {
        if let IrreducibleNode::Quote(exprs) = value.node {
            exprs
        } else {
            let mut exprs = Vec::new();
            let Irreducible { span, ty, .. } = value;
            let res = self.make_irr(&mut exprs, value);

            ExprSeq {
                exprs,
                branch: Branch {
                    node: BranchNode::Return(res),
                    span,
                    ty,
                },
                span,
                ty,
            }
        }
    }

    fn make_irr(&mut self, within: &mut Vec<Expr>, value: Irreducible) -> Value {
        let node = match value.node {
            IrreducibleNode::Invalid => ValueNode::Invalid,
            IrreducibleNode::Integer(i) => {
                self.check_int_range(value.span, i, &value.ty);
                ValueNode::Int(i)
            }

            IrreducibleNode::Lambda(params, body) => {
                let name = self.names.fresh(value.span, None);
                self.context.add(name, value.ty);

                let body = self.promote(*body);

                within.push(Expr {
                    node: ExprNode::Function { name, params, body },
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
                if let BranchNode::Return(ref value) = exprs.branch.node {
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
