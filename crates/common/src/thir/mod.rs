mod because;
mod constraint;
mod context;
mod tree;
mod types;

pub use because::Because;
pub use constraint::Constraint;
pub use context::{Context, TypeOrSchema};
pub use tree::{Decls, Expr, ExprNode, Pat, PatNode, ValueDef};
pub use types::{instantiate, Type, UniVar};

use std::collections::HashMap;

#[derive(Debug)]
pub struct TypeckResult {
    pub context: Context,
    pub decls: Decls<Type>,
    pub subst: HashMap<UniVar, Type>,
    pub constraints: Vec<Constraint>,
}
