mod because;
mod constraint;
mod context;
mod pretty;
mod tree;
mod types;

pub use self::pretty::pretty_type;
pub use because::Because;
pub use constraint::Constraint;
pub use context::{merge_insts, Context, TypeOrSchema};
pub use tree::{Decls, Expr, ExprNode, Pat, PatNode, ValueDef};
pub use types::{Mutability, Type, UniVar};

use std::collections::HashMap;

use crate::names::Name;

#[derive(Debug)]
pub struct TypeckResult {
    pub context: Context,
    pub decls: Decls<Type>,
    pub subst: HashMap<UniVar, (HashMap<Name, Type>, Type)>,
    pub constraints: Vec<Constraint>,
}
