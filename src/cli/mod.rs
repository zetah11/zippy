mod diagnostic;
mod format;

#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::fs;

use log::LevelFilter;
use simple_logger::SimpleLogger;
use zippy_common::messages::Messages;
use zippy_common::source::project::module_name_from_source;
use zippy_common::source::Project;
use zippy_frontend::names::declare::declared_names;
use zippy_frontend::parser::get_ast;

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
    let prettier = Prettier::new(&database);

    let mut asts: HashMap<_, Vec<_>> = HashMap::new();

    for source in database.sources.iter() {
        let source = *source;

        let ast = get_ast(&database, source);
        let source_name = *ast.source(&database).name(&database);
        let module_name = module_name_from_source(&database, source_name);

        asts.entry(module_name).or_default().push(ast);

        for message in get_ast::accumulated::<Messages>(&database, source) {
            print_diagnostic(&database, &prettier, message)?;
        }
    }

    let modules: Vec<_> = asts
        .into_iter()
        .map(|(name, sources)| zippy_frontend::ast::Module::new(&database, name, sources))
        .collect();

    for module in modules {
        let _ = declared_names(&database, module);

        for message in declared_names::accumulated::<Messages>(&database, module) {
            print_diagnostic(&database, &prettier, message)?;
        }
    }

    Ok(())
}
