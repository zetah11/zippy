use super::Resolver;
use crate::unresolved::{Type, TypeNode};

impl Resolver<'_> {
    pub fn declare_type(&mut self, ty: &Type) {
        match &ty.node {
            TypeNode::Name(_) => {}

            TypeNode::Fun(t, u) | TypeNode::Product(t, u) => {
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
