use super::Evaluator;
use crate::mir::{Expr, ExprNode, Pat, PatNode};
use crate::Driver;

impl<'a, D: Driver> Evaluator<'a, D> {
    /// Bind a value to a pattern, which may require reducing the expression.
    pub fn bind(&mut self, pat: Pat, value: Expr) {
        match pat.node {
            PatNode::Wildcard | PatNode::Invalid => {}
            PatNode::Name(name) => {
                self.set(name, value);
            }

            PatNode::Tuple(a, b) => match value.node {
                ExprNode::Tuple(x, y) => {
                    self.bind(*a, *x);
                    self.bind(*b, *y);
                }

                _ => todo!(),
            },
        }
    }
}
