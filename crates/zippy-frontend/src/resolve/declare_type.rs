use zippy_common::hir::{Type, TypeNode};

use super::Resolver;

impl Resolver {
    pub fn declare_type(&mut self, ty: &Type) {
        match &ty.node {
            TypeNode::Name(_) => {}
            TypeNode::Fun(t, u) | TypeNode::Prod(t, u) => {
                self.declare_type(t);
                self.declare_type(u);
            }

            TypeNode::Range(lo, hi) => {
                self.declare_expr(lo);
                self.declare_expr(hi);
            }

            TypeNode::Type | TypeNode::Wildcard | TypeNode::Invalid => {}
        }
    }
}
