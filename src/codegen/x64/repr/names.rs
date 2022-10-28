use std::collections::HashMap;

use crate::resolve::names as resolve;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Name(pub(in super::super) usize);

#[derive(Debug, Default)]
pub struct Names {
    names: HashMap<Name, resolve::Name>,
    inverse: HashMap<resolve::Name, Name>,
    id: usize,
}

impl Names {
    pub fn new() -> Self {
        Self {
            names: HashMap::new(),
            inverse: HashMap::new(),
            id: 0,
        }
    }

    pub fn add(&mut self, name: resolve::Name) -> Name {
        if let Some(res) = self.inverse.get(&name) {
            *res
        } else {
            let res = Name(self.id);
            self.id += 1;
            self.names.insert(res, name);
            self.inverse.insert(name, res);
            res
        }
    }

    pub fn get(&self, name: &Name) -> &resolve::Name {
        self.names.get(name).unwrap()
    }
}