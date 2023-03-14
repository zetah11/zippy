//! Various representations for item names encountered during compilation.
//!
//! Note that zippy makes a distinction between *declarations* and *expressions*
//! and this distinction shows up with the names as well. Declarations are
//! allowed to be declared in any order and may refer to themselves, while
//! expressions follow a "top-down" approach. This means that names may not be
//! shadowed within the same "level" of declarations (because that would be
//! ambiguous), while they are allowed to be shadowed within expressions
//! (because they are read in a specific order).
//!
//! Thus, any name declared in a declarative section will eventually become an
//! [`ItemName`] while any name declared in an expression will become a
//! [`LocalName`]. The latter contains a `scope` id which is used to
//! disambiguate it from other locals with the same name. Both contain an
//! optional reference to a "parent" name, which is the name of the item or
//! local that contains it.
//!
//! Finally, note that some declarations don't have a convenient name to
//! identify them. For instance:
//!
//! ```zippy
//! let _: Int =
//!   let x = 5
//!   x
//! ```
//!
//! As such, there is a third kind of name - the [`UnnamableName`] - which is
//! identified by a [`Span`] instead of a name.

use crate::source::Span;
use crate::Db;

/// This is simply an interned string intended to make it easier and fast to
/// compare names. It should not be used as an identifier by itself, however.
#[salsa::interned]
pub struct RawName {
    #[return_ref]
    pub text: String,
}

/// A name of an item in some unordered declarative region.
#[salsa::interned]
pub struct ItemName {
    pub parent: Option<DeclarableName>,
    pub name: RawName,
}

/// A name of a local in some ordered, scoped region.
#[salsa::interned]
pub struct LocalName {
    pub parent: Option<DeclarableName>,
    pub name: RawName,
    pub scope: usize,
}

/// The name of some item whose pattern contains no names, and so can only be
/// identified by its span.
#[salsa::interned]
pub struct UnnamableName {
    pub kind: UnnamableNameKind,
    pub parent: Option<DeclarableName>,
    pub span: Span,
}

/// How did this unnamable name come to be? Through an anonymous object? A
/// lambda?
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum UnnamableNameKind {
    /// An unnamed entry
    Entry,

    /// An unnamed function such as a lambda.
    Lambda,

    /// Some pattern that's not just a single name.
    Pattern,
}

/// Represents any kind of name that may be referred to.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Name {
    Item(ItemName),
    Local(LocalName),
}

impl From<ItemName> for Name {
    fn from(name: ItemName) -> Self {
        Self::Item(name)
    }
}

impl From<LocalName> for Name {
    fn from(name: LocalName) -> Self {
        Self::Local(name)
    }
}

/// Represents any kind of name that can be created with some kind of
/// declaration or binding. This is different from a [referrable name](Name)
/// because it also includes unnamable names (which, by definition, cannot be
/// referred to). These may be used as the parent of any item.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DeclarableName {
    Item(ItemName),
    Local(LocalName),
    Unnamable(UnnamableName),
}

impl DeclarableName {
    pub fn to_name(self) -> Option<Name> {
        match self {
            Self::Item(item) => Some(Name::Item(item)),
            Self::Local(item) => Some(Name::Local(item)),
            Self::Unnamable(_) => None,
        }
    }

    pub fn parent(&self, db: &dyn Db) -> Option<DeclarableName> {
        match self {
            DeclarableName::Item(item) => item.parent(db),
            DeclarableName::Local(local) => local.parent(db),
            DeclarableName::Unnamable(name) => name.parent(db),
        }
    }
}

impl From<Name> for DeclarableName {
    fn from(value: Name) -> Self {
        match value {
            Name::Item(name) => DeclarableName::Item(name),
            Name::Local(name) => DeclarableName::Local(name),
        }
    }
}

impl From<ItemName> for DeclarableName {
    fn from(name: ItemName) -> Self {
        Self::Item(name)
    }
}

impl From<LocalName> for DeclarableName {
    fn from(name: LocalName) -> Self {
        Self::Local(name)
    }
}

impl From<UnnamableName> for DeclarableName {
    fn from(name: UnnamableName) -> Self {
        Self::Unnamable(name)
    }
}
