use common::names::Name;

use super::state::Frame;
use super::value::Operation;
use super::Interpreter;

/// A [`Place`] refers to a particular instruction or branch. Each place is
/// associated with a [`Name`] which uniquely identifies a straight-line block
/// of code. [`Place::Instruction`] refers to a particular instruction by its
/// index within that block, while [`Place::Branch`] refers to the unique branch
/// in a block.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Place {
    Instruction(Name, usize),
    Branch(Name),
}

impl Place {
    pub fn name(&self) -> Name {
        match self {
            Self::Instruction(name, _) => *name,
            Self::Branch(name) => *name,
        }
    }
}

impl Interpreter<'_> {
    /// Get a mutable reference to the current frame. Panics if there is none.
    pub fn get_frame_mut(&mut self) -> &mut Frame {
        self.frames.last_mut().unwrap()
    }

    pub fn get_operation(&mut self, place: &Place) -> Option<Operation> {
        let name = place.name();
        // O Polonius, Polonius, wherefore art thou Polonius?
        let block = self.blocks.get(&name)?;

        Some(match place {
            Place::Branch(_) => Operation::Branch(block.branch.clone()),
            Place::Instruction(_, index) => {
                Operation::Statement(block.stmts.get(*index).unwrap().clone())
            }
        })
    }

    pub fn get_place(&self) -> Option<Place> {
        self.frames.last().map(|frame| frame.place)
    }

    pub fn place_of(&self, name: &Name) -> Option<Place> {
        let block = self.blocks.get(name)?;
        if block.stmts.is_empty() {
            Some(Place::Branch(*name))
        } else {
            Some(Place::Instruction(*name, 0))
        }
    }
}
