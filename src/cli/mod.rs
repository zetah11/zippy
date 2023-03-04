mod diagnostic;
mod format;

#[cfg(test)]
mod tests;

use std::fs;

use log::LevelFilter;
use simple_logger::SimpleLogger;
use zippy_common::messages::Messages;
use zippy_common::source::Project;
use zippy_frontend::ast;
use zippy_frontend::names::declare::declared_names;
use zippy_frontend::parser::parse;

use crate::pretty::Prettier;
use crate::project::{source_name_from_path, FsProject, DEFAULT_ROOT_NAME};
use crate::{project, Database};

use self::diagnostic::print_diagnostic;

/// Perform checks on the project.
pub fn check() -> anyhow::Result<()> {
    SimpleLogger::new()
        .with_module_level("salsa_2022", LevelFilter::Warn)
        .init()
        .unwrap();

    let cwd = std::env::current_dir()?;
    let mut database = Database::new();

    let project_name = cwd
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| DEFAULT_ROOT_NAME.to_string());
    let project = Project::new(&database, project_name);
    let project = FsProject::new(project).with_root(&cwd);

    for path in project::get_project_sources(&cwd) {
        let content = fs::read_to_string(&path)?;
        let name = source_name_from_path(&database, Some(&project), &path);
        database.write_source(path, name, content);
    }

    let database = database.with_root(cwd);

    let mut messages = Vec::new();
    let sources: Vec<_> = database.sources.iter().map(|source| *source).collect();
    let asts = parse(&database, &mut messages, &sources);

    for (name, sources) in asts {
        let module = ast::Module::new(&database, name, sources);
        database.ast_modules.insert(name, module);
    }

    for module in database.ast_modules.iter() {
        let _ = declared_names(&database, *module);
        messages.extend(declared_names::accumulated::<Messages>(&database, *module));
    }

    let prettier = Prettier::new(&database);
    for message in messages {
        print_diagnostic(&database, &prettier, message)?;
    }

    Ok(())
}
