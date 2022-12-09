use std::collections::HashMap;

use common::names::Name;

use super::value::ReducedValue;
use super::Interpreter;

#[derive(Debug, Default)]
pub struct Env {
    values: HashMap<Name, ReducedValue>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn add(&mut self, name: Name, value: ReducedValue) {
        assert!(self.values.insert(name, value).is_none());
    }

    pub fn get(&self, name: &Name) -> Option<&ReducedValue> {
        self.values.get(name)
    }
}

impl Interpreter<'_> {
    /// Bind the given name to a value. Panics if there is no stack frame to
    /// bind it in, or if the name has already been added in the current stack
    /// frame.
    pub fn bind(&mut self, name: Name, value: ReducedValue) {
        self.get_frame_mut().add(name, value);
    }
}
