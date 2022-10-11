pub mod pretty;

use std::collections::HashMap;

pub use tree::{Decls, Expr, ExprNode, ExprSeq, Value, ValueDef};
pub use types::Types;

mod tree;
mod types;

use crate::resolve::names::Name;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TypeId(usize);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Type {
    Range(i64, i64),
    Fun(TypeId, TypeId),
    Product(TypeId, TypeId),
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

    pub fn get(&self, name: &Name) -> TypeId {
        *self.names.get(name).unwrap()
    }
}
