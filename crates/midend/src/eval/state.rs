use common::names::Name;

use super::environment::Env;
use super::place::Place;
use super::value::ReducedValue;

#[derive(Debug)]
pub struct Frame {
    pub place: Place,
    pub return_names: Option<Vec<Name>>,
    pub env: Env,
}

impl Frame {
    pub fn new(place: Place) -> Self {
        Self {
            place,
            return_names: None,
            env: Env::new(),
        }
    }

    /// Bind a value to a name. Panics if the name is already bound.
    pub fn add(&mut self, name: Name, value: ReducedValue) {
        self.env.add(name, value)
    }

    pub fn get(&self, name: &Name) -> Option<&ReducedValue> {
        self.env.get(name)
    }
}
