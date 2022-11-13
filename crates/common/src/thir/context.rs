use std::collections::HashMap;

use super::{Type, UniVar};
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

    pub fn instantiate(&mut self, schema: &TypeOrSchema) -> (Type, Vec<UniVar>) {
        match schema {
            TypeOrSchema::Type(ty) => (ty.clone(), vec![]),
            TypeOrSchema::Schema(params, ty) => {
                let vars: Vec<_> = (0..params.len()).map(|_| self.fresh()).collect();
                let mapping = params.iter().copied().zip(vars.iter().copied()).collect();

                let ty = super::instantiate(&mapping, ty);

                (ty, vars)
            }
        }
    }

    pub fn fresh(&mut self) -> UniVar {
        let id = UniVar(self.curr_var);
        self.curr_var += 1;
        id
    }
}

impl IntoIterator for Context {
    type IntoIter = std::collections::hash_map::IntoIter<Name, Type>;
    type Item = (Name, Type);

    fn into_iter(self) -> Self::IntoIter {
        todo!()
    }
}
