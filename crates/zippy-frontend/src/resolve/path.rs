use zippy_common::hir::BindId;

use crate::unresolved::Name;

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum NamePart {
    Source(Name),
    Scope(BindId),
}

/// A path is a name and all of its containing names. Effectively, this gives a
/// *path* from the root of the code to this name.
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct Path(pub Vec<NamePart>, pub NamePart);
