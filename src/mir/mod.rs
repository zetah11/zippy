pub mod pretty;

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
