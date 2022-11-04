use std::collections::HashMap;

use super::{Type, UniVar};
use crate::names::Name;

#[derive(Debug, Default)]
pub struct Context {
    names: HashMap<Name, Type>,
    curr_var: usize,
}

impl Context {
    pub fn new() -> Self {
        Self {
            names: HashMap::new(),
            curr_var: 0,
        }
    }

    pub fn add(&mut self, name: Name, ty: Type) {
        assert!(self.names.insert(name, ty).is_none());
    }

    pub fn get(&self, name: &Name) -> &Type {
        self.names.get(name).unwrap()
    }

    pub fn fresh(&mut self) -> UniVar {
        let id = UniVar(self.curr_var);
        self.curr_var += 1;
        id
    }
}

impl IntoIterator for Context {
    type IntoIter = std::collections::hash_map::IntoIter<Name, Type>;
    type Item = (Name, Type);

    fn into_iter(self) -> Self::IntoIter {
        self.names.into_iter()
    }
}