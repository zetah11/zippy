pub mod pretty;

use std::collections::HashMap;

pub use check::check;
pub use discover::discover;
pub use tree::{Block, Branch, BranchNode, Decls, Statement, StmtNode, Value, ValueDef, ValueNode};
pub use types::Types;

mod check;
mod discover;
mod tree;
mod types;

use crate::names::Name;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TypeId(usize);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Type {
    Range(i64, i64),
    Fun(Vec<TypeId>, Vec<TypeId>),
    Product(Vec<TypeId>),
    Invalid,
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

    pub fn replace(&mut self, name: Name, ty: TypeId) {
        assert!(self.names.insert(name, ty).is_some());
    }

    pub fn get(&self, name: &Name) -> TypeId {
        *self.names.get(name).unwrap()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Name, &TypeId)> {
        self.names.iter()
    }
}
