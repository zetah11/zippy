pub mod ast;
pub mod check;
pub mod checked;
pub mod clarify;
pub mod dependencies;
pub mod flattened;
pub mod flattening;
pub mod names;
pub mod parser;
pub mod resolved;

mod messages;

pub trait Db: salsa::DbWithJar<Jar> + salsa::DbWithJar<zippy_common::Jar> {}

impl<Database> Db for Database where
    Database: salsa::DbWithJar<Jar> + salsa::DbWithJar<zippy_common::Jar>
{
}

#[salsa::jar(db = Db)]
pub struct Jar(
    crate::ast::AstSource,
    crate::check::get_bound,
    crate::check::Bound,
    crate::flattened::Module,
    crate::flattening::flatten_module,
    crate::names::declare::declared_names,
    crate::names::resolve::resolve_module,
    crate::parser::get_ast,
    crate::parser::get_tokens,
    crate::resolved::Module,
);
