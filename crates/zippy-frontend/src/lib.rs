pub mod ast;
pub mod parser;

mod messages;

pub trait Db: salsa::DbWithJar<Jar> + salsa::DbWithJar<zippy_common::Jar> {}

impl<Database> Db for Database where
    Database: salsa::DbWithJar<Jar> + salsa::DbWithJar<zippy_common::Jar>
{
}

#[salsa::jar(db = Db)]
pub struct Jar(crate::ast::AstSource, crate::parser::get_ast);
