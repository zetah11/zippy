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

    let mut modules: HashMap<_, Vec<_>> = HashMap::new();

    for path in project::get_project_sources(&cwd) {
        let content = fs::read_to_string(&path)?;
        let name = source_name_from_path(&database, Some(&project), &path);
        let source = database.write_source(path, name, content);

        let module_name = module_name_from_source(&database, name);
        modules.entry(module_name).or_default().push(source);
    }

    for (name, sources) in modules {
        database.write_module(name, sources);
    }

    let database = database.with_root(cwd);

    let mut messages = Vec::new();

    for module in database.modules.iter() {
        let _ = declared_names(&database, *module);
        messages.extend(declared_names::accumulated::<Messages>(&database, *module));
    }

    let prettier = Prettier::new(&database).with_full_name(true);
    for message in messages {
        print_diagnostic(&database, &prettier, message)?;
    }

    Ok(())
}
