use super::{Project, SourceName};
use crate::names::{DeclarableName, ItemName, RawName};
use crate::Db;

#[salsa::tracked]
pub fn module_name_from_source(db: &dyn Db, source: SourceName) -> ItemName {
    let root = project_root(db, source.project(db));
    let mut name = root;

    for part in source.parts(db) {
        let part = RawName::new(db, part.clone());
        name = ItemName::new(db, Some(DeclarableName::Item(name)), part);
    }

    name
}

#[salsa::tracked]
pub fn project_root(db: &dyn Db, project: Project) -> ItemName {
    let root = RawName::new(db, project.name(db).clone());
    ItemName::new(db, None, root)
}
