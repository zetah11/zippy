mod because;
mod coerce;
mod constraint;
mod context;
mod defs;
mod pretty;
mod tree;
mod types;

pub use self::because::Because;
pub use self::coerce::{Coercion, CoercionId, Coercions};
pub use self::constraint::Constraint;
pub use self::context::{merge_insts, Context, TypeOrSchema};
pub use self::defs::Definitions;
pub use self::pretty::{pretty_type, PrettyMap};
pub use self::tree::{Decls, Expr, ExprNode, Pat, PatNode, TypeDef, ValueDef};
pub use self::types::{Mutability, Type, UniVar};

use std::collections::HashMap;

use crate::names::Name;

#[derive(Debug)]
pub struct TypeckResult {
    pub coercions: Coercions,
    pub context: Context,
    pub defs: Definitions,
    pub decls: Decls<Type>,
    pub subst: HashMap<UniVar, (HashMap<Name, Type>, Type)>,
    pub constraints: Vec<Constraint>,
}
