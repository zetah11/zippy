use std::collections::HashMap;

use super::{Mutability, Type, UniVar};
use crate::names::Name;

#[derive(Clone, Debug)]
pub enum TypeOrSchema {
    Type(Type),
    Schema(Vec<Name>, Type),
}

#[derive(Debug, Default)]
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
                let mapping = params.iter().copied().zip(vars.iter().copied()).collect();

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
}

impl IntoIterator for Context {
    type IntoIter = std::collections::hash_map::IntoIter<Name, Type>;
    type Item = (Name, Type);

    fn into_iter(self) -> Self::IntoIter {
        todo!()
    }
}
