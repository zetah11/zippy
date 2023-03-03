mod diagnostic;
mod format;

#[cfg(test)]
mod tests;

use log::LevelFilter;
use simple_logger::SimpleLogger;
use zippy_common::messages::Messages;
use zippy_frontend::parser::get_ast;

use crate::{project, Database};

use self::diagnostic::print_diagnostic;

/// Perform checks on the project.
pub fn check() -> anyhow::Result<()> {
    SimpleLogger::new()
        .with_module_level("salsa_2022", LevelFilter::Warn)
        .init()
        .unwrap();

    let cwd = std::env::current_dir()?;
    let database = Database::new();

    for source in project::get_project_sources(&cwd) {
        let content = database.read_source(source.clone())?;
        database.add_source(source, content);
    }

    let database = database.with_root(cwd);

    for source in database.sources.iter() {
        let source = *source;
        let _ = get_ast(&database, source);
        for message in get_ast::accumulated::<Messages>(&database, source) {
            print_diagnostic(&database, message)?;
        }
    }

    Ok(())
}
