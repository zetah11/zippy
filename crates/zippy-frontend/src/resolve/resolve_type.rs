use super::Resolver;
use crate::resolved::{Type, TypeNode};
use crate::unresolved;

impl Resolver<'_> {
    pub fn resolve_type(&mut self, ty: unresolved::Type) -> Type {
        let node = match ty.node {
            unresolved::TypeNode::Name(name) => match self.lookup(ty.span, name) {
                Some(name) => TypeNode::Name(name),
                None => TypeNode::Invalid,
            },

            unresolved::TypeNode::Product(t, u) => {
                let t = Box::new(self.resolve_type(*t));
                let u = Box::new(self.resolve_type(*u));

                TypeNode::Product(t, u)
            }

            unresolved::TypeNode::Fun(t, u) => {
                let t = Box::new(self.resolve_type(*t));
                let u = Box::new(self.resolve_type(*u));

                TypeNode::Fun(t, u)
            }

            unresolved::TypeNode::Range(lo, hi) => {
                let lo = Box::new(self.resolve_expr(*lo));
                let hi = Box::new(self.resolve_expr(*hi));

                TypeNode::Range(lo, hi)
            }

            unresolved::TypeNode::Type => TypeNode::Type,
            unresolved::TypeNode::Wildcard => TypeNode::Wildcard,
            unresolved::TypeNode::Invalid => TypeNode::Invalid,
        };

        Type {
            node,
            span: ty.span,
        }
    }
}
