mod files;
mod modules;

pub(crate) use self::modules::{source_name_from_path, DEFAULT_ROOT_NAME};

use zippy_common::source::Project;

use std::path::{Path, PathBuf};

use log::{error, warn};

use self::files::files;

/// Represents a project within a filesystem.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct FsProject {
    pub project: Project,

    /// The root directory of the project
    pub root_dir: Option<PathBuf>,
}

impl FsProject {
    pub fn new(project: Project) -> Self {
        Self {
            project,
            root_dir: None,
        }
    }

    pub fn with_root(self, root_dir: impl Into<PathBuf>) -> Self {
        Self {
            root_dir: Some(root_dir.into()),
            ..self
        }
    }
}

/// Get the source names of every source that is part of the project in the
/// current directory.
pub fn get_project_sources(root: impl AsRef<Path>) -> Vec<PathBuf> {
    let files = match files(root) {
        Ok(files) => files,
        Err(e) => {
            error!("Unable to find project sources:");
            error!("{e}");
            return Vec::new();
        }
    };

    let mut result = Vec::new();

    for file in files {
        let path = match file {
            Ok(path) => path,
            Err(e) => {
                warn!("Error with file:");
                warn!("{e}");
                continue;
            }
        };

        let Some(ext) = path.extension() else {
            continue;
        };

        let name = match ext.to_str() {
            Some("z") => path,
            _ => continue,
        };

        result.push(name);
    }

    result
}
