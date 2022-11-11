use std::collections::HashMap;

use crate::names::Name;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TypeId(usize);

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Type {
    Range(i64, i64),
    Product(Vec<TypeId>),
    Fun(Vec<TypeId>, Vec<TypeId>),
}

#[derive(Debug, Default)]
pub struct Context {
    names: HashMap<Name, TypeId>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            names: HashMap::new(),
        }
    }

    pub fn add(&mut self, name: Name, ty: TypeId) {
        assert!(self.names.insert(name, ty).is_none());
    }

    pub fn get(&self, name: &Name) -> TypeId {
        *self.names.get(name).unwrap()
    }
}

#[derive(Debug, Default)]
pub struct Types {
    types: Vec<Type>,
}

impl Types {
    pub fn new() -> Self {
        Self { types: Vec::new() }
    }

    pub fn add(&mut self, ty: Type) -> TypeId {
        if let Some(id) = self.types.iter().position(|other| other == &ty) {
            TypeId(id)
        } else {
            let id = TypeId(self.types.len());
            self.types.push(ty);
            id
        }
    }

    pub fn get(&self, ty: &TypeId) -> &Type {
        self.types.get(ty.0).unwrap()
    }

    pub fn is_function(&self, ty: &TypeId) -> bool {
        matches!(self.get(ty), Type::Fun(..))
    }
}
