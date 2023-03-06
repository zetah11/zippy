pub mod components;
pub mod invalid;
pub mod messages;
pub mod names;
pub mod source;

use self::names::ItemName;
use self::source::Module;

pub trait Db: salsa::DbWithJar<Jar> {
    /// Get the module with the given name, if any.
    fn get_module(&self, name: &ItemName) -> Option<Module>;
}

#[salsa::jar(db = Db)]
pub struct Jar(
    crate::messages::Messages,
    crate::names::ItemName,
    crate::names::LocalName,
    crate::names::RawName,
    crate::names::UnnamableName,
    crate::source::Module,
    crate::source::Project,
    crate::source::Source,
    crate::source::SourceName,
    crate::source::project::module_name_from_source,
    crate::source::project::project_root,
);
