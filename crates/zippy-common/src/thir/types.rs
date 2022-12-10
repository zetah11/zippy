use std::collections::HashMap;

use crate::names::Name;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UniVar(pub(super) usize);

/// Determines the mutability of a type variable. Mutable type variables can be "assigned" a substitution, while an
/// immutable one can only be replaced with its substitution, if it has one. Outside a function, any type variables in
/// that function's signature is immutable (meaning external code won't affect it), while inside, those type variables
/// may be mutable. That way, only code inside a definition affects the inference of it.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Mutability {
    Immutable,
    Mutable,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Type {
    Name(Name),

    Range(i64, i64),
    Fun(Box<Type>, Box<Type>),

    Product(Box<Type>, Box<Type>),

    Instantiated(Box<Type>, HashMap<Name, Type>),
    Var(Mutability, UniVar),
    Number,

    Invalid,
}

impl Type {
    pub fn mutable(var: UniVar) -> Self {
        Self::Var(Mutability::Mutable, var)
    }

    pub fn immutable(var: UniVar) -> Self {
        Self::Var(Mutability::Immutable, var)
    }

    pub fn make_mutability(&self, mutability: Mutability) -> Type {
        match self {
            Type::Name(name) => Type::Name(*name),

            Type::Range(lo, hi) => Type::Range(*lo, *hi),

            Type::Fun(t, u) => {
                let t = Box::new(t.make_mutability(mutability));
                let u = Box::new(u.make_mutability(mutability));
                Type::Fun(t, u)
            }

            Type::Product(t, u) => {
                let t = Box::new(t.make_mutability(mutability));
                let u = Box::new(u.make_mutability(mutability));
                Type::Product(t, u)
            }

            Type::Instantiated(ty, insts) => {
                let ty = Box::new(ty.make_mutability(mutability));
                Type::Instantiated(ty, insts.clone())
            }

            Type::Var(_, var) => Type::Var(mutability, *var),

            Type::Number => Type::Number,
            Type::Invalid => Type::Invalid,
        }
    }
}

pub fn instantiate(mapping: &HashMap<Name, Type>, ty: &Type) -> Type {
    match ty {
        Type::Name(name) => match mapping.get(name) {
            Some(ty) => instantiate(mapping, ty),
            None => Type::Name(*name),
        },

        Type::Product(t, u) => {
            let t = Box::new(instantiate(mapping, t));
            let u = Box::new(instantiate(mapping, u));
            Type::Product(t, u)
        }

        Type::Fun(t, u) => {
            let t = Box::new(instantiate(mapping, t));
            let u = Box::new(instantiate(mapping, u));
            Type::Fun(t, u)
        }

        Type::Instantiated(ty, prev_mapping) => {
            for key in prev_mapping.keys() {
                assert!(!mapping.contains_key(key));
            }

            for key in mapping.keys() {
                assert!(!prev_mapping.contains_key(key));
            }

            let ty = Box::new(instantiate(mapping, ty));
            Type::Instantiated(ty, prev_mapping.clone())
        }

        Type::Range(lo, hi) => Type::Range(*lo, *hi),
        Type::Number => Type::Number,
        Type::Var(mutable, var) => Type::Var(*mutable, *var),
        Type::Invalid => Type::Invalid,
    }
}
