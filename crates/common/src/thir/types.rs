use std::collections::HashMap;

use crate::names::Name;

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct UniVar(pub(super) usize);

#[derive(Clone, Debug)]
pub enum Type {
    Name(Name),

    Range(i64, i64),
    Fun(Box<Type>, Box<Type>),

    Product(Box<Type>, Box<Type>),

    Var(UniVar),
    Number,

    Invalid,
}

pub fn instantiate(mapping: &HashMap<Name, UniVar>, ty: &Type) -> Type {
    match ty {
        Type::Name(name) => match mapping.get(name) {
            Some(var) => Type::Var(*var),
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

        Type::Range(lo, hi) => Type::Range(*lo, *hi),
        Type::Number => Type::Number,
        Type::Var(var) => Type::Var(*var),
        Type::Invalid => Type::Invalid,
    }
}
