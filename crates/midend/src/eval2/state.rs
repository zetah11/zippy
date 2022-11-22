use common::names::Name;

use super::code::{Place, Value};
use super::env::Env;

#[derive(Debug)]
pub struct Frame {
    /// The current location of the interpreter.
    pub place: Place,
    pub env: Env,
}

#[derive(Debug, Default)]
pub struct State {
    pub frames: Vec<Frame>,
    pub globals: Env,
}

impl State {
    /// Create an empty [`State`].
    pub fn new() -> Self {
        Self {
            frames: Vec::new(),
            globals: Env::new(),
        }
    }

    /// Create a new [`State`] which includes the globals of this one.
    pub fn split(&self) -> Self {
        Self {
            frames: Vec::new(),
            globals: self.globals.clone(),
        }
    }

    /// Merge the globals from another state with this one. Prefers the value of a name in `other` if both have the
    /// value defined and they don't match.
    pub fn merge(&mut self, other: Self) {
        self.globals.extend(other.globals);
    }

    /// Push a new `frame` onto the frame stack.
    pub fn enter(&mut self, frame: Frame) {
        self.frames.push(frame);
    }

    /// Return the current frame on the frame stack. Panics if the stack is empty.
    pub fn exit(&mut self) -> Frame {
        self.frames.pop().expect("unmatched `exit`")
    }

    /// Add a name-value binding to the current frame. Panics if the stack is empty.
    pub fn add(&mut self, name: Name, value: Value) {
        self.frames.last_mut().unwrap().env.add(name, value);
    }

    pub fn add_global(&mut self, name: Name, value: Value) {
        self.globals.add(name, value);
    }

    // Search the stack and the globals for the value associated with `name`. Returns `None` if the name is not bound.
    pub fn get(&self, name: &Name) -> Option<&Value> {
        for frame in self.frames.iter().rev() {
            if let Some(value) = frame.env.get(name) {
                return Some(value);
            }
        }

        if let Some(value) = self.globals.get(name) {
            Some(value)
        } else {
            None
        }
    }

    /// Get the current frame. Returns `None` if the stack is empty.
    pub fn current(&self) -> Option<&Frame> {
        self.frames.last()
    }

    /// Get a mutable reference to the current frame. Returns `None` if the stack is empty.
    pub fn current_mut(&mut self) -> Option<&mut Frame> {
        self.frames.last_mut()
    }
}
