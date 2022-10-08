use super::Evaluator;
use crate::mir::{Expr, Pat, PatNode};
use crate::Driver;

impl<'a, D: Driver> Evaluator<'a, D> {
    /// Bind a value to a pattern, which may require reducing the expression.
    pub fn bind(&mut self, pat: Pat, value: Expr) {
        match pat.node {
            PatNode::Wildcard | PatNode::Invalid => {}
            PatNode::Name(name) => {
                self.set(name, value);
            }
        }
    }
}
