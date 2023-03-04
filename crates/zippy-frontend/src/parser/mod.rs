mod abstraction;
mod cst;
mod messages;
mod parse;
mod tokens;

use std::collections::HashMap;

use zippy_common::messages::{Message, Messages};
use zippy_common::names::ItemName;
use zippy_common::source::project::module_name_from_source;
use zippy_common::source::Source;

use self::abstraction::abstract_item;
use self::parse::Parser;
use self::tokens::TokenIter;
use crate::ast::AstSource;
use crate::Db;

/// Produce a map of every module and the sources it consists of.
pub fn parse(
    db: &dyn Db,
    messages: &mut Vec<Message>,
    sources: &[Source],
) -> HashMap<ItemName, Vec<AstSource>> {
    let zdb = <dyn Db as salsa::DbWithJar<zippy_common::Jar>>::as_jar_db(db);
    let mut modules: HashMap<_, Vec<_>> = HashMap::new();

    for source in sources {
        let ast = get_ast(db, *source);
        let source_name = ast.source(db).name(zdb);
        let module_name = module_name_from_source(zdb, *source_name);

        modules.entry(module_name).or_default().push(ast);

        messages.extend(get_ast::accumulated::<Messages>(db, *source));
    }

    modules
}

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
