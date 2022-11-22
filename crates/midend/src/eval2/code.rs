use common::names::Name;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum InstructionPlace {
    Execute,
    Bind,
}

#[derive(Clone, Copy, Debug)]
pub enum Place {
    Instruction(Name, usize, InstructionPlace),
    Branch(Name),
}

impl Place {
    pub fn name(&self) -> Name {
        match self {
            Place::Branch(name) => *name,
            Place::Instruction(name, ..) => *name,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Value {
    /// A single integer value.
    Int(i64),

    /// Some erroneous value.
    Invalid,

    /// A function referenced by its name.
    Function(Name),
}
