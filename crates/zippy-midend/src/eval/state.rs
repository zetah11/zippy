use zippy_common::names::Name;
use zippy_common::Driver;

use super::environment::Env;
use super::place::Place;
use super::value::ReducedValue;
use super::Interpreter;

#[derive(Debug)]
pub struct Frame {
    pub place: Place,
    pub return_names: Option<Vec<Name>>,
    pub env: Env,
    pub frame_index: usize,
}

impl Frame {
    /// Bind a value to a name. Panics if the name is already bound.
    pub fn add(&mut self, name: Name, value: ReducedValue) {
        self.env.add(name, value)
    }

    pub fn get(&self, name: &Name) -> Option<&ReducedValue> {
        self.env.get(name)
    }
}

impl<D: Driver> Interpreter<'_, D> {
    pub(super) fn new_frame(&mut self, place: Place) -> Frame {
        Frame {
            place,
            return_names: None,
            env: Env::new(),
            frame_index: self.new_frame_index(),
        }
    }
}
