use super::{Name, Type};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Context {
    names: HashMap<Name, Type>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            names: HashMap::new(),
        }
    }

    pub fn add(&mut self, name: Name, ty: Type) {
        assert!(self.names.insert(name, ty).is_none());
    }

    pub fn get(&self, name: &Name) -> &Type {
        self.names.get(name).unwrap()
    }
}
