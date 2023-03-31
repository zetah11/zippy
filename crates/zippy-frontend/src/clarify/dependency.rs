use zippy_common::names::{ItemName, LocalName, Name};

use crate::checked::ItemIndex;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Dependency {
    /// This expression directly depends on a local name.
    LocalRef(LocalName),

    /// This expression directly depends on an item name.
    ItemRef(ItemName),

    /// This expression defines or depends on a particular item, so it depends
    /// on any name that that item depends on.
    Item(ItemIndex),
}

impl From<LocalName> for Dependency {
    fn from(name: LocalName) -> Self {
        Dependency::LocalRef(name)
    }
}

impl From<ItemName> for Dependency {
    fn from(name: ItemName) -> Self {
        Dependency::ItemRef(name)
    }
}

impl From<Name> for Dependency {
    fn from(name: Name) -> Self {
        match name {
            Name::Item(name) => name.into(),
            Name::Local(name) => name.into(),
        }
    }
}
