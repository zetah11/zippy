use zippy_common::kinds;

use crate::resolved::{Type, TypeNode};

use super::Kinder;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UniVar(usize);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Kind {
    Type,

    Function(Box<Kind>, Box<Kind>),
    Product(Box<Kind>, Box<Kind>),

    Var(UniVar),
    Invalid,
}

impl Kinder<'_> {
    pub fn fresh(&mut self) -> UniVar {
        let var = UniVar(self.counter);
        self.counter += 1;
        var
    }

    pub fn kind_from_type(&mut self, ty: Type) -> Kind {
        match ty.node {
            TypeNode::Type => Kind::Type,

            TypeNode::Fun(t, u) => {
                let t = Box::new(self.kind_from_type(*t));
                let u = Box::new(self.kind_from_type(*u));

                Kind::Function(t, u)
            }

            TypeNode::Product(t, u) => {
                let t = Box::new(self.kind_from_type(*t));
                let u = Box::new(self.kind_from_type(*u));

                Kind::Product(t, u)
            }

            TypeNode::Wildcard => Kind::Var(self.fresh()),

            TypeNode::Invalid => Kind::Invalid,

            _ => {
                self.messages.at(ty.span).kick_not_a_kind();
                Kind::Invalid
            }
        }
    }

    pub fn substitute(&self, mut kind: Kind) -> kinds::Kind {
        loop {
            return match kind {
                Kind::Type => kinds::Kind::Type,

                Kind::Function(a, b) => {
                    let a = Box::new(self.substitute(*a));
                    let b = Box::new(self.substitute(*b));

                    kinds::Kind::Function(a, b)
                }

                Kind::Product(a, b) => {
                    let a = Box::new(self.substitute(*a));
                    let b = Box::new(self.substitute(*b));

                    kinds::Kind::Product(a, b)
                }

                Kind::Var(var) => {
                    // Ambiguous or un-unified vars should be generalized by
                    // the kind checker, so this should never fail.
                    kind = self.subst.get(&var).unwrap().clone();
                    continue;
                }

                Kind::Invalid => kinds::Kind::Invalid,
            };
        }
    }
}
