pub mod components;
pub mod definitions;
pub mod kick;
pub mod lex;
pub mod parse;
pub mod resolve;
pub mod tyck2;

mod resolved;
mod unresolved;

use salsa::DbWithJar;
use zippy_common::names::{Name, Names};
use zippy_common::thir::TypeckResult;
use zippy_common::Driver;

#[derive(Debug)]
pub struct ParseResult {
    pub checked: TypeckResult,
    pub names: Names,
    pub entry: Option<Name>,
}

pub fn parse(driver: &mut impl Driver, source: String, file: usize) -> ParseResult {
    let db = Database::default();
    let program = SourceProgram::new(&db, source, file);

    let tokens = lex::lex(&db, program);
    let decls = parse::parse(&db, tokens);
    let _decls = resolve::resolve(&db, decls);

    todo!()

    // let tyckres = tyck::typeck(driver, &mut names, decls);

    // ParseResult {
    //     checked: tyckres,
    //     names,
    //     entry,
    // }
}

#[salsa::accumulator]
pub struct MessageAccumulator(zippy_common::message::Diagnostic);

#[salsa::input]
pub struct SourceProgram {
    #[return_ref]
    pub text: String,
    pub id: usize,
}

#[salsa::jar(db = Db)]
pub struct Jar(
    crate::SourceProgram,
    crate::MessageAccumulator,
    crate::resolved::Decls,
    crate::unresolved::Name,
    crate::unresolved::Decls,
    crate::lex::Tokens,
    crate::lex::lex,
    crate::parse::parse,
    crate::resolve::resolve,
    crate::components::Components,
    crate::components::components,
    crate::definitions::type_definitions,
    crate::kick::Kinds,
    crate::kick::kindck,
    crate::tyck2::typeck,
);

pub trait Db: DbWithJar<Jar> + DbWithJar<zippy_common::Jar> {}

impl<DB> Db for DB where DB: DbWithJar<Jar> + DbWithJar<zippy_common::Jar> {}

#[derive(Default)]
#[salsa::db(crate::Jar, zippy_common::Jar)]
pub(crate) struct Database {
    storage: salsa::Storage<Self>,
}

impl salsa::Database for Database {}

impl salsa::ParallelDatabase for Database {
    fn snapshot(&self) -> salsa::Snapshot<Self> {
        salsa::Snapshot::new(Self {
            storage: self.storage.snapshot(),
        })
    }
}
