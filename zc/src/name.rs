//! Names uniquely identify certain items, and come in a few shapes.
//!
//! - **Contextual names** are most of the literal names in source code, whose full path is
//!   determined by context (such as scope and visible imports)
//! - **Fully qualified names** are names that contain the entire path from the relevant package
//!   root through all containing modules and items. These unambiguously refer either to one item
//!   or none.
//!
//! Name declaration is the process of finding which names are defined in a program. Name
//! resolution is the process of finding the fully qualified name for all names.
//!
//! Internally, resolved names are stored as simple integers, which makes them fast and lightweight
//! to pass around and identify things by. Getting a resolved name is done by creating a
//! [`QualifiedName`] and passing it to [`NameStore::from_qualified`].

#[cfg(test)]
mod tests;

use bimap::BiMap;

/// A qualified name represents a complete, unambiguous path from the root through pacakges and
/// modules and other containing items with its final name at the end.
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum QualifiedName {
    /// The name of an item that exists in the root. This is typically used for pacakges.
    Root(String),

    /// The name of something contained in another item.
    Contained {
        /// The resolved name of this items container. If the item lives in a module, this is the
        /// name of that module, for instance.
        container: Name,

        /// The contextual name for this item.
        name: String,
    },
}

/// `Name`s are lightweight types which represents unambiguous items. These are created and can be
/// used with a [`NameStore`] to get or store [`QualifiedName`]s.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct Name(usize);

/// Stores mappings between [`QualifiedName`]s and their lightweight [`Name`]s. In general, only
/// one [`NameStore`] should ever be used with one compiler - using multiple serves little purposes
/// and can lead to bugs as certain functions assume it is the only one being interacted with.
#[derive(Debug, Default)]
pub struct NameStore {
    names: BiMap<Name, QualifiedName>,

    /// The next free name id. This number is monotonically increasing, which means insertions are
    /// as fast as the underlying map.
    curr_id: usize,
}

impl NameStore {
    /// Create a new, empty [`NameStore`].
    pub fn new() -> Self {
        Self {
            names: BiMap::new(),
            curr_id: 0,
        }
    }

    /// Add a name to this store, and return its representation. If the qualified name has already
    /// been added, that name is returned instead.
    pub fn add_qualified(&mut self, qualified: QualifiedName) -> Name {
        if let Some(name) = self.names.get_by_right(&qualified) {
            return *name;
        }

        let name = Name(self.curr_id);
        self.curr_id += 1;
        self.names.insert(name, qualified);
        name
    }

    /// Get the name assosciated with this qualified name.
    pub fn get_name(&self, qualified: &QualifiedName) -> Option<Name> {
        self.names.get_by_right(qualified).copied()
    }

    /// Get the qualified name assosciated with this name.
    pub fn get_qualified(&self, name: &Name) -> Option<&QualifiedName> {
        self.names.get_by_left(name)
    }

    /// Returns `true` if this store has the given [`Name`]. This only returns `false` if the name
    /// has been removed (or if multiple stores are used, although that is most likely a mistake).
    pub fn has_name(&self, name: &Name) -> bool {
        self.names.contains_left(name)
    }

    /// Returns `true` if this store has the given [`QualifiedName`].
    pub fn has_qualified(&self, qualified: &QualifiedName) -> bool {
        self.names.contains_right(qualified)
    }

    /// Remove the given name and all other names that reference it. Panics if `name` has already
    /// been removed before.
    pub fn remove_name(&mut self, name: &Name) {
        self.names.remove_by_left(name).unwrap();

        let to_remove: Vec<_> = self
            .names
            .iter()
            .filter_map(|(k, v)| match v {
                QualifiedName::Contained { container, .. } if container == name => Some(*k),
                _ => None,
            })
            .collect();

        for name in to_remove {
            self.remove_name(&name);
        }
    }

    /// Remove the given qualified name and all other names that reference it. Does nothing if
    /// `qualified` does not exist in the store (as opposed to [`NameStore::remove_name`], which
    /// panics).
    pub fn remove_qualified(&mut self, qualified: &QualifiedName) {
        if let Some(name) = self.get_name(qualified) {
            self.remove_name(&name);
        }
    }
}
