//! This module contains representations of fully qualified names, which are
//! globally unambiguous. A qualified name is effectively a literal name (like
//! a string appearing in the source or a generated one) together with a path
//! from the root through all the names that "contain" this one.

use std::collections::HashMap;

use crate::Db;

/// A literal name or a generated one.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum NamePart {
    Source(String),

    /// A compiler-generated name, identified by some automatically generated
    /// id.
    Generated(usize),
}

/// A globally unambiguous name.
#[salsa::interned]
pub struct Name {
    pub path: Option<Name>,
    #[return_ref]
    pub name: NamePart,
}

/// A name generator is responsible for generating unique names. The generated
/// names are local to a given scope.
pub struct NameGenerator {
    counters: HashMap<Option<Name>, usize>,
}

impl NameGenerator {
    /// Create an empty [`NameGenerator`].
    #[must_use]
    pub fn new() -> Self {
        Self {
            counters: HashMap::new(),
        }
    }

    /// Generate a name that is unique in the current local scope (`context`).
    #[must_use]
    pub fn fresh(&mut self, db: &dyn Db, context: Option<Name>) -> Name {
        let id = self.make_id(context);
        Name::new(db, context, NamePart::Generated(id))
    }

    fn make_id(&mut self, context: Option<Name>) -> usize {
        let counter = self.counters.entry(context).or_default();
        let id = *counter;
        *counter += 1;
        id
    }
}
