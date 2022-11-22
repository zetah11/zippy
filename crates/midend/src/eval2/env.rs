use std::collections::HashMap;
use std::iter::{Extend, IntoIterator};

use common::names::Name;

use super::code::Value;

#[derive(Clone, Debug, Default)]
pub struct Env {
    map: HashMap<Name, Value>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn add(&mut self, name: Name, value: Value) {
        assert!(self.map.insert(name, value).is_none());
    }

    pub fn get(&self, name: &Name) -> Option<&Value> {
        self.map.get(name)
    }
}

impl Extend<(Name, Value)> for Env {
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = (Name, Value)>,
    {
        self.map.extend(iter)
    }
}

impl IntoIterator for Env {
    type IntoIter = std::collections::hash_map::IntoIter<Name, Value>;
    type Item = (Name, Value);

    fn into_iter(self) -> Self::IntoIter {
        self.map.into_iter()
    }
}
