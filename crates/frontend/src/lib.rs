pub mod lex;
pub mod parse;
pub mod resolve;
pub mod tyck;

use common::names::{Name, Names};
use common::thir::TypeckResult;
use common::Driver;

use resolve::ResolveRes;

#[derive(Debug)]
pub struct ParseResult {
    pub checked: TypeckResult,
    pub names: Names,
    pub entry: Option<Name>,
}

pub fn parse(driver: &mut impl Driver, source: String, file: usize) -> ParseResult {
    let tokens = lex::lex(driver, source, file);
    let decls = parse::parse(driver, tokens, file);
    let ResolveRes {
        decls,
        names,
        entry,
    } = resolve::resolve(driver, decls);
    let tyckres = tyck::typeck(driver, &names, decls);

    ParseResult {
        checked: tyckres,
        names,
        entry,
    }
}
