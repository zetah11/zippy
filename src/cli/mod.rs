mod diagnostic;
mod dot;
mod format;

#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::fs;

use log::LevelFilter;
use simple_logger::SimpleLogger;
use zippy_common::messages::Messages;
use zippy_common::source::Project;
use zippy_frontend::check;
use zippy_frontend::dependencies::get_dependencies;

use self::diagnostic::print_diagnostic;
use self::dot::GraphViz;
use crate::database::Database;
use crate::pretty::Prettier;
use crate::project;
use crate::project::{source_name_from_path, FsProject, DEFAULT_ROOT_NAME};

/// Perform checks on the project.
pub fn check(dot: bool) -> anyhow::Result<()> {
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
    let prettier = Prettier::new(&database).with_full_name(false);

    let _checked = check::check(&database, &mut messages, database.get_modules());

    if dot {
        let mut all_deps = HashMap::new();
        for module in database.get_modules() {
            let dependencies = get_dependencies(&database, module);
            messages.extend(get_dependencies::accumulated::<Messages>(&database, module));

            for (name, depends) in dependencies.dependencies(&database) {
                assert!(all_deps.insert(*name, depends.clone()).is_none());
            }
        }

        let graph = std::fs::File::create("dependencies.dot")?;
        let mut writer = std::io::BufWriter::new(graph);
        GraphViz::new(&database, &prettier, all_deps).render(&mut writer)?;
    }

    for message in messages {
        print_diagnostic(&database, Some(&project), &prettier, message)?;
    }

    Ok(())
}
