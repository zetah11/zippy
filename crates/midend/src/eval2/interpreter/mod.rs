mod run;
mod step;
mod value;

use std::collections::HashMap;

use common::mir::{Block, Decls, StaticValue, StaticValueNode, Types};
use common::names::{Name, Names};

use super::code::{InstructionPlace, Place, Value};
use super::env::Env;
use super::result::Error;
use super::state::{Frame, State, StateAction};

#[derive(Debug)]
pub struct Interpreter<'a> {
    decls: Decls,
    worklist: Vec<State>,
    return_values: Vec<Value>,

    /// A block is some straight-line sequence of instructions followed by a branch (like a return or a jump), indexed
    /// by name.
    blocks: HashMap<Name, Block>,

    /// Stores the names of parameters for functions. The actual function body is in a `block`.
    functions: HashMap<Name, Vec<Name>>,

    names: &'a Names,
    types: &'a Types,
}

impl<'a> Interpreter<'a> {
    pub fn new(names: &'a Names, types: &'a Types, decls: Decls) -> Self {
        Self {
            decls,
            worklist: Vec::new(),
            return_values: Vec::new(),

            blocks: HashMap::new(),
            functions: HashMap::new(),

            names,
            types,
        }
    }

    pub fn entry(&mut self, name: Name) {
        let mut state = State::new(StateAction::Nothing);
        let place = self.place_of(&name);
        let frame = Frame {
            env: Env::new(),
            place,
        };

        state.enter(frame);
        self.worklist.push(state);
    }

    pub fn returned(&self) -> &[Value] {
        &self.return_values
    }

    /// Get the block that contains `place`, or `None` if the block has not been initialized.
    fn block_of(&self, place: &Place) -> Option<&Block> {
        self.blocks.get(&place.name())
    }

    /// Get the block that containts `place`, initializing it from a top level declaration if not already initialized.
    /// Panics if there is no top level declaration for the given place, or if the place points to a global value.
    fn block_of_or_top_level(&mut self, place: &Place) -> &Block {
        let name = place.name();
        if self.blocks.contains_key(&name) {
            self.blocks.get(&name).unwrap()
        } else {
            self.top_level(name);
            self.blocks.get(&name).unwrap()
        }
    }

    /// Get the current state, or `None` if the worklist is empty.
    fn current(&self) -> Option<&State> {
        self.worklist.last()
    }

    /// Get a mutable reference to the current state, or `None` if the worklist is empty.
    fn current_mut(&mut self) -> Option<&mut State> {
        self.worklist.last_mut()
    }

    fn has_top_level(&mut self, name: &Name) -> bool {
        self.decls.defs.iter().any(|def| def.name == *name)
            || self.decls.functions.contains_key(name)
            || self.decls.values.contains_key(name)
    }

    // Merge the two topmost states. If there's only one left, removes it from the worklist.
    fn merge_down(&mut self) {
        if self.worklist.len() < 2 {
            // todo: do something about the state action here too?
            let _ = self.worklist.pop();
        } else {
            let mut top = self.worklist.pop().unwrap();

            match top.action {
                StateAction::Nothing => {}
                StateAction::StoreGlobal(name) => {
                    assert!(self.return_values.len() == 1);
                    top.add_global(name, self.return_values.pop().unwrap());
                }
            }

            self.worklist.last_mut().unwrap().merge(top);
        }
    }

    fn make_function(&mut self, name: Name, params: Vec<Name>, block: Block) {
        assert!(self.functions.insert(name, params).is_none());
        assert!(self.blocks.insert(name, block).is_none());
    }

    fn make_static(&mut self, name: Name, value: StaticValue) {
        match value.node {
            StaticValueNode::Int(i) => {
                if let Some(state) = self.current_mut() {
                    state.add_global(name, Value::Int(i));
                } else {
                    todo!()
                }
            }

            StaticValueNode::LateInit(block) => {
                assert!(self.blocks.insert(name, block).is_none());
            }
        }
    }

    fn top_level(&mut self, name: Name) {
        if let Some(def) = self.decls.defs.iter().find(|def| def.name == name) {
            assert!(self.blocks.insert(name, def.bind.clone()).is_none());
        } else if let Some(value) = self.decls.values.get(&name) {
            self.make_static(name, value.clone());
        } else if let Some((params, block)) = self.decls.functions.get(&name) {
            self.make_function(name, params.clone(), block.clone());
        } else {
            unreachable!();
        }
    }

    fn place(&self) -> Option<Place> {
        Some(self.current()?.current()?.place)
    }

    /// Get the place for the function or value with the given `name`. Adds it as a top-level if it hasn't already been.
    fn place_of(&mut self, name: &Name) -> Place {
        if !self.blocks.contains_key(name) {
            self.top_level(*name);
        }

        match self.blocks.get(name) {
            Some(block) if block.stmts.is_empty() => Place::Branch(*name),

            Some(_) => Place::Instruction(*name, 0, InstructionPlace::Execute),

            None => unreachable!(),
        }
    }
}
