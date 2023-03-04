mod diagnostic;
mod format;

#[cfg(test)]
mod tests;

use std::fs;

use log::LevelFilter;
use simple_logger::SimpleLogger;
use zippy_common::messages::Messages;
use zippy_common::source::Project;
use zippy_frontend::names::declare::declared_names;

use crate::database::Database;
use crate::pretty::Prettier;
use crate::project;
use crate::project::{source_name_from_path, FsProject, DEFAULT_ROOT_NAME};

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

    let sources = project::get_project_sources(&cwd)
        .into_iter()
        .filter_map(|path| {
            let content = fs::read_to_string(&path).ok()?;
            let name = source_name_from_path(&database, Some(&project), &path);
            Some((path, name, content))
        })
        .collect();

    database.init_sources(sources);

    let mut messages = Vec::new();

    for module in database.get_modules() {
        let _ = declared_names(&database, module);
        messages.extend(declared_names::accumulated::<Messages>(&database, module));
    }

    let prettier = Prettier::new(&database).with_full_name(false);
    for message in messages {
        print_diagnostic(&database, Some(&project), &prettier, message)?;
    }

    Ok(())
}
