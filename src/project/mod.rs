mod files;

use std::path::{Path, PathBuf};

use log::{error, warn};

use self::files::files;

/// The name of a source, which for the language server is just a path to a
/// file.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct SourceName(PathBuf);

impl SourceName {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self(path.as_ref().into())
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

/// Get the source names of every source that is part of the project in the
/// current directory.
pub fn get_project_sources(root: impl AsRef<Path>) -> Vec<SourceName> {
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
            Some("z") => SourceName::new(path),
            _ => continue,
        };

        result.push(name);
    }

    result
}
