pub mod ast;
pub mod dependencies;
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
    crate::dependencies::get_dependencies,
    crate::dependencies::ModuleDependencies,
    crate::names::declare::declared_names,
    crate::names::resolve::resolve_module,
    crate::parser::get_ast,
    crate::resolved::Module,
);
