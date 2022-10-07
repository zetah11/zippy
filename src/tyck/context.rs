use super::{Name, Type, UniVar};
use std::collections::HashMap;

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
