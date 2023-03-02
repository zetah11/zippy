mod abstraction;
mod cst;
mod messages;
mod parse;
mod tokens;

use zippy_common::messages::Messages;
use zippy_common::source::Source;

use self::abstraction::abstract_item;
use self::parse::Parser;
use self::tokens::TokenIter;
use crate::ast::AstSource;
use crate::Db;

#[salsa::tracked]
pub fn get_ast(db: &dyn Db, source: Source) -> AstSource {
    let zdb = <dyn Db as salsa::DbWithJar<zippy_common::Jar>>::as_jar_db(db);

    let tokens = TokenIter::new(source, source.content(zdb)).filter_map(|result| match result {
        Ok(token) => Some(token),
        Err(message) => {
            Messages::push(zdb, message);
            None
        }
    });

    let mut parser = Parser::new(db, source, tokens);
    let items = parser.parse_items();
    let items = items
        .into_iter()
        .filter_map(|item| abstract_item(db, item))
        .collect();

    AstSource::new(db, source, items)
}
