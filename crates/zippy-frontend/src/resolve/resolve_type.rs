use zippy_common::hir::{Type, TypeNode};
use zippy_common::names::{Actual, Name};

use super::Resolver;

impl Resolver {
    pub fn resolve_type(&mut self, ty: Type) -> Type<Name> {
        let node = match ty.node {
            TypeNode::Name(name) => match self.lookup_name(ty.span, Actual::Lit(name)) {
                Some(name) => TypeNode::Name(name),
                None => TypeNode::Invalid,
            },

            TypeNode::Prod(t, u) => {
                let t = Box::new(self.resolve_type(*t));
                let u = Box::new(self.resolve_type(*u));

                TypeNode::Prod(t, u)
            }

            TypeNode::Fun(t, u) => {
                let t = Box::new(self.resolve_type(*t));
                let u = Box::new(self.resolve_type(*u));

                TypeNode::Fun(t, u)
            }

            TypeNode::Range(lo, hi) => TypeNode::Range(lo, hi),
            TypeNode::Wildcard => TypeNode::Wildcard,
            TypeNode::Invalid => TypeNode::Invalid,
        };

        Type {
            node,
            span: ty.span,
        }
    }
}
