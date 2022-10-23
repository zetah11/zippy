mod instruction;
mod register;

use std::collections::HashMap;

use super::repr::Name;

#[derive(Clone, Debug)]
struct Relocation {
    pub kind: RelocationKind,
    pub at: usize,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum RelocationKind {
    /// 32-bit relocation, relative to current instruction
    Relative,

    /// 32-bit relocation, relative to next instruction
    RelativeNext,

    /// 64-bit absolute relocation.
    Absolute,
}

#[derive(Debug)]
struct Encoder {
    relocations: HashMap<Name, Vec<Relocation>>,
    code: Vec<u8>,
}
