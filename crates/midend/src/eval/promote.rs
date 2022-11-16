//! Promotion converts an [`Irreducible`](super::Irreducible) back to an `ExprSeq`.

use common::names::Name;

use super::{Irreducible, IrreducibleNode, Lowerer};
use crate::mir::{Block, Branch, BranchNode, Statement, StmtNode};
use crate::mir::{Value, ValueNode};
use crate::Driver;

impl<D: Driver> Lowerer<'_, D> {
    pub fn promote(&mut self, ctx: Name, value: Irreducible) -> Block {
        if let IrreducibleNode::Quote(exprs) = value.node {
            exprs
        } else {
            let mut exprs = Vec::new();
            let Irreducible { span, ty, .. } = value;
            let res = self.make_irr(&mut exprs, ctx, value);

            Block {
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

    fn make_irr(
        &mut self,
        within: &mut Vec<Statement>,
        ctx: Name,
        value: Irreducible,
    ) -> Vec<Value> {
        let node = match value.node {
            IrreducibleNode::Invalid => ValueNode::Invalid,
            IrreducibleNode::Integer(i) => {
                self.check_int_range(value.span, i, &value.ty);
                ValueNode::Int(i)
            }

            IrreducibleNode::Lambda(params, body) => {
                let name = self.names.fresh(value.span, ctx);
                self.context.add(name, value.ty);

                let body = self.promote(name, *body);

                within.push(Statement {
                    node: StmtNode::Function { name, params, body },
                    span: value.span,
                    ty: value.ty,
                });

                ValueNode::Name(name)
            }

            IrreducibleNode::Tuple(values) => {
                return values
                    .into_iter()
                    .flat_map(|value| self.make_irr(within, ctx, value))
                    .collect();
            }

            IrreducibleNode::Quote(exprs) => {
                if let BranchNode::Return(ref values) = exprs.branch.node {
                    let values = values.clone();
                    within.extend(exprs.exprs);
                    return values;
                } else {
                    unreachable!()
                }
            }
        };

        vec![Value {
            node,
            span: value.span,
            ty: value.ty,
        }]
    }
}
