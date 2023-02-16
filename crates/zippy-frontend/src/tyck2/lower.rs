use zippy_common::hir2::{Mutability, Type};

use super::Typer;
use crate::resolved;

impl Typer<'_> {
    pub fn lower_type(&mut self, ty: &resolved::Type) -> Type {
        match &ty.node {
            resolved::TypeNode::Name(name) => Type::Name(*name),
            resolved::TypeNode::Range(lo, hi) => Type::Range(*lo, *hi),

            resolved::TypeNode::Fun(t, u) => {
                let t = Box::new(self.lower_type(t));
                let u = Box::new(self.lower_type(u));
                Type::Fun(t, u)
            }

            resolved::TypeNode::Product(t, u) => {
                let t = Box::new(self.lower_type(t));
                let u = Box::new(self.lower_type(u));
                Type::Product(t, u)
            }

            resolved::TypeNode::Type => Type::Type,
            resolved::TypeNode::Number => Type::Number,
            resolved::TypeNode::Wildcard => Type::Var(Mutability::Mutable, self.context.fresh()),
            resolved::TypeNode::Invalid => Type::Invalid,
        }
    }
}
