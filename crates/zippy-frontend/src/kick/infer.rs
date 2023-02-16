use super::kinds::Kind;
use super::Kinder;
use crate::resolved::{Type, TypeNode};

impl Kinder<'_> {
    pub fn infer(&mut self, ty: &Type) -> Kind {
        match &ty.node {
            TypeNode::Name(name) => self.context.get(name).unwrap().clone(),

            TypeNode::Range(..) => Kind::Type,

            TypeNode::Fun(t, u) => {
                let a = self.infer(t);
                let b = self.infer(u);

                // (->) : type -> type -> type
                self.unify(t.span, a, Kind::Type);

                self.unify(u.span, b, Kind::Type);

                Kind::Type
            }

            TypeNode::Product(t, u) => {
                let a = self.infer(t);
                let b = self.infer(u);

                // (*) : type -> type -> type
                self.unify(t.span, a, Kind::Type);

                self.unify(u.span, b, Kind::Type);

                Kind::Type
            }

            TypeNode::Number => Kind::Type,
            TypeNode::Wildcard | TypeNode::Invalid => Kind::Var(self.fresh()),

            TypeNode::Type => todo!(),
        }
    }
}
