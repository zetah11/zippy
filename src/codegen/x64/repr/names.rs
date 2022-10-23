use std::collections::HashMap;

use crate::resolve::names as resolve;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Name(usize);

#[derive(Debug, Default)]
pub struct Names {
    names: HashMap<Name, resolve::Name>,
    id: usize,
}

impl Names {
    pub fn new() -> Self {
        Self {
            names: HashMap::new(),
            id: 0,
        }
    }

    pub fn add(&mut self, name: resolve::Name) -> Name {
        let res = Name(self.id);
        self.id += 1;
        self.names.insert(res, name);
        res
    }

    pub fn get(&self, name: &Name) -> &resolve::Name {
        self.names.get(name).unwrap()
    }
}
