use std::collections::HashMap;

use super::{Mutability, Type, UniVar};
use crate::names2::Name;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TypeOrSchema {
    Type(Type),
    Schema(Vec<Name>, Type),
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Context {
    names: HashMap<Name, TypeOrSchema>,
    curr_var: usize,
}

impl Context {
    pub fn new() -> Self {
        Self {
            names: HashMap::new(),
            curr_var: 0,
        }
    }

    pub fn add(&mut self, name: Name, ty: Type) {
        assert!(self.names.insert(name, TypeOrSchema::Type(ty)).is_none());
    }

    pub fn add_schema(&mut self, name: Name, params: Vec<Name>, ty: Type) {
        assert!(self
            .names
            .insert(name, TypeOrSchema::Schema(params, ty))
            .is_none());
    }

    pub fn get(&self, name: &Name) -> &TypeOrSchema {
        self.names.get(name).unwrap()
    }

    pub fn fresh(&mut self) -> UniVar {
        let id = UniVar(self.curr_var);
        self.curr_var += 1;
        id
    }

    pub fn instantiate(&mut self, schema: &TypeOrSchema) -> (Type, Vec<UniVar>) {
        match schema {
            TypeOrSchema::Type(ty) => (ty.clone(), vec![]),
            TypeOrSchema::Schema(params, ty) => {
                let vars: Vec<_> = (0..params.len()).map(|_| self.fresh()).collect();
                let mapping = params
                    .iter()
                    .copied()
                    .zip(vars.iter().map(|var| Type::mutable(*var)))
                    .collect();

                let ty = super::types::instantiate(&mapping, ty);
                let ty = Type::Instantiated(Box::new(ty), mapping);

                (ty, vars)
            }
        }
    }

    /// Modify the mutability of the type bound to this name.
    pub fn make_mutability(&mut self, name: &Name, mutability: Mutability) {
        let schema = match self.names.remove(name) {
            Some(TypeOrSchema::Type(ty)) => TypeOrSchema::Type(ty.make_mutability(mutability)),
            Some(TypeOrSchema::Schema(params, ty)) => {
                TypeOrSchema::Schema(params, ty.make_mutability(mutability))
            }
            None => unreachable!(),
        };

        self.names.insert(*name, schema);
    }

    /// Get an iterator over all of the polymorphic names in this context.
    pub fn polymorphic_names(&self) -> impl Iterator<Item = Name> + '_ {
        self.names.iter().filter_map(|(name, ty)| match ty {
            TypeOrSchema::Schema(..) => Some(*name),
            _ => None,
        })
    }
}

pub fn merge_insts(a: &HashMap<Name, Type>, b: &HashMap<Name, Type>) -> HashMap<Name, Type> {
    let mut res = HashMap::with_capacity(a.len() + b.len());

    // As we merge, it might be that there are duplicates in `a` or `b`. I *think* if
    // this happens, then the unifier must have unified the two types such that it doesn't
    // matter which one we actually use.
    for (name, ty) in a.iter() {
        res.insert(*name, ty.clone());
    }

    for (name, ty) in b.iter() {
        res.insert(*name, ty.clone());
    }
    res
}
