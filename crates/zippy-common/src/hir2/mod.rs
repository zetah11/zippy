mod because;
mod coerce;
mod constraint;
mod context;
mod definitions;
mod pretty;
mod tree;
mod types;

pub use self::because::Because;
pub use self::coerce::{Coercion, CoercionId, Coercions};
pub use self::constraint::Constraint;
pub use self::context::{merge_insts, Context, TypeOrSchema};
pub use self::definitions::Definitions;
pub use self::pretty::{pretty_type, PrettyMap};
pub use self::tree::{Decls, Expr, ExprNode, Pat, PatNode, ValueDef};
pub use self::types::{Mutability, Type, UniVar};

use std::collections::HashMap;

use crate::names2::Name;

#[salsa::tracked]
pub struct TypeckResult {
    pub coercions: Coercions,
    pub context: Context,
    pub decls: Decls,
    pub subst: HashMap<UniVar, (HashMap<Name, Type>, Type)>,
    pub constraints: Vec<Constraint>,
}
