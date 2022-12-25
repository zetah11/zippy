//! Keeps track of type definitions.

use std::collections::HashMap;

use super::Type;
use crate::names::Name;

#[derive(Debug, Default)]
pub struct Definitions {
    types: HashMap<Name, Type>,
}

impl Definitions {
    pub fn new() -> Self {
        Self {
            types: HashMap::new(),
        }
    }

    pub fn add(&mut self, name: Name, ty: Type) {
        assert!(self.types.insert(name, ty).is_none());
    }

    pub fn get(&self, name: &Name) -> Option<&Type> {
        self.types.get(name)
    }

    pub fn has(&self, name: &Name) -> bool {
        self.types.contains_key(name)
    }

    /// Returns `Some(true)` if the given type is a numeric type, `Some(false)`
    /// if it is not, and `None` if the given type has not been defined.
    pub fn is_numeric(&self, name: &Name) -> Option<bool> {
        Some(match self.get(name)? {
            Type::Range(..) => true,
            Type::Name(name) => self.is_numeric(name)?,

            Type::Instantiated(..) | Type::Number => unreachable!(),

            _ => false,
        })
    }
}

pub struct DefsIter {
    types: std::collections::hash_map::IntoIter<Name, Type>,
}

impl Iterator for DefsIter {
    type Item = (Name, Type);

    fn next(&mut self) -> Option<Self::Item> {
        self.types.next()
    }
}

impl IntoIterator for Definitions {
    type IntoIter = DefsIter;
    type Item = (Name, Type);

    fn into_iter(self) -> Self::IntoIter {
        DefsIter {
            types: self.types.into_iter(),
        }
    }
}
