use std::ffi::OsStr;
use std::path::{Component, Path};

use log::warn;
use zippy_common::source::{Project, SourceName};

use super::FsProject;
use crate::Database;

pub const DEFAULT_ROOT_NAME: &str = "Root";

/// Get the name of the module that this file is a part of.
///
/// Sources may be split into directories, where each directory is a single
/// module. Nested directories are nested modules. If the database has no
/// project root, then the module name is the name of the file itself.
///
/// Directory- and filenames are converted into title case.
///
/// (cursed warning: paths are cursed, therefore this function is cursed)
pub(crate) fn source_name_from_path(
    db: &Database,
    project: Option<&FsProject>,
    path: impl AsRef<Path>,
) -> SourceName {
    let path = path.as_ref();
    if let Some(project) = project {
        if let Some(ref root) = project.root_dir {
            if let Ok(path) = path.strip_prefix(root) {
                multi_file_module_name(db, project, path)
            } else {
                single_file_module_name(db, path)
            }
        } else {
            single_file_module_name(db, path)
        }
    } else {
        single_file_module_name(db, path)
    }
}

fn multi_file_module_name(db: &Database, project: &FsProject, relative_path: &Path) -> SourceName {
    let mut parts = Vec::new();

    for component in relative_path.components() {
        match component {
            Component::Normal(part) => parts.push(kebab_to_title(part)),

            // Lord help me if I have to deal with these
            Component::CurDir | Component::ParentDir => unreachable!(),
            Component::Prefix(_) | Component::RootDir => unreachable!(),
        }
    }

    SourceName::new(db, project.project, parts)
}

fn single_file_module_name(db: &Database, path: &Path) -> SourceName {
    // path.file_prefix() I await your arrival
    let name = if let Some(name) = path.file_stem() {
        kebab_to_title(name)
    } else if let Some(name) = path.file_name() {
        kebab_to_title(name)
    } else {
        DEFAULT_ROOT_NAME.to_string()
    };

    let project = Project::new(db, name);
    SourceName::new(db, project, Vec::new())
}

/// Convert some path component in kebab case to a title case version.
/// Specifically, the first letter and letters following a dash is capitalized,
/// and the dashes themselves are skipped.
///
/// Because an [`OsStr`] may be *very* funky, any invalid characters are just
/// skipped completely. This should maybe produce an error if it happens?
fn kebab_to_title(name: &OsStr) -> String {
    let name = name.to_string_lossy();
    let mut result = String::with_capacity(name.len());
    let mut capitalize = true;

    for c in name.chars() {
        match c {
            '-' => {
                capitalize = true;
            }

            char::REPLACEMENT_CHARACTER => {
                warn!("file or folder {name} contains invalid characters");
            }

            c if capitalize => {
                result.extend(c.to_uppercase());
                capitalize = false;
            }

            c => {
                result.push(c);
            }
        }
    }

    result.shrink_to_fit();
    result
}
