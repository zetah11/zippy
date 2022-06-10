//! Names uniquely identify certain items, and come in a few shapes.
//!
//! - **Contextual names** are most of the literal names in source code, whose
//!   full path is determined by context (such as scope and visible imports)
//! - **Fully qualified names** are names that contain the entire path from the
//!   relevant package root through all containing modules and items. These
//!   unambiguously refer either to one item or none.

pub use interner::{NameInterner, NameInternerStorage};

use salsa::{InternId, InternKey};

mod interner {
    #![allow(missing_docs)] // no docs for `lookup_*` methods

    use super::{Name, NameData};

    /// Responsible for mapping more heavyweight names (with their paths) to the
    /// lightweight [`Name`](super::Name) ids.
    #[salsa::query_group(NameInternerStorage)]
    pub trait NameInterner {
        /// Intern a fully qualified name to a lightweight id.
        #[salsa::interned]
        fn intern_name(&self, name: NameData) -> Name;
    }
}

/// A qualified name represents a complete, unambiguous path from the root
/// through pacakges and modules and other containing items with its final name
/// at the end.
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum NameData {
    /// A top-level item.
    Root(String),

    /// Some name contained within another scope
    Child(Name, String),
}

/// `Name`s are lightweight types which unambiguously represent an item.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct Name(InternId);

impl InternKey for Name {
    fn from_intern_id(v: InternId) -> Self {
        Self(v)
    }

    fn as_intern_id(&self) -> InternId {
        self.0
    }
}
