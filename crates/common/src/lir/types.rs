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

    pub fn offsetof(&self, ty: &TypeId, ndx: usize) -> usize {
        match self.get(ty) {
            Type::Product(ties) => {
                assert!(ties.len() > ndx);
                ties.iter().take(ndx).map(|ty| self.sizeof(ty)).sum()
            }

            Type::Fun(..) | Type::Range(..) => unreachable!(),
        }
    }

    /// Get the size of the given type in bytes.
    pub fn sizeof(&self, ty: &TypeId) -> usize {
        match self.get(ty) {
            Type::Range(lo, hi) => {
                let range = if 0 < *lo {
                    *hi as usize
                } else if 0 > *hi {
                    -(*lo as i128) as usize
                } else {
                    (hi - lo) as usize
                };

                ((range as f64).log2() / 8.0).ceil() as usize
            }

            Type::Product(ties) => ties.iter().map(|ty| self.sizeof(ty)).sum(),

            Type::Fun(..) => 8,
        }
    }
}
