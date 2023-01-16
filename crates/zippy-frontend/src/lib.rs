pub mod lex;
pub mod parse;
pub mod resolve;
pub mod tyck;

use zippy_common::names::{Name, Names};
use zippy_common::thir::TypeckResult;
use zippy_common::Driver;

use resolve::ResolveRes;

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
    let decls = parse::parse(driver, tokens.tokens(&db).iter().cloned(), file);
    let ResolveRes {
        decls,
        mut names,
        entry,
    } = resolve::resolve(driver, decls);
    let tyckres = tyck::typeck(driver, &mut names, decls);

    ParseResult {
        checked: tyckres,
        names,
        entry,
    }
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
pub struct Jar(SourceProgram, MessageAccumulator, lex::Tokens, lex::lex);

pub trait Db: salsa::DbWithJar<Jar> {}

impl<DB> Db for DB where DB: salsa::DbWithJar<Jar> {}

#[derive(Default)]
#[salsa::db(crate::Jar)]
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
