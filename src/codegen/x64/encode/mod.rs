mod block;
mod instruction;
mod procedure;
mod program;
mod register;
mod relocation;

use std::collections::HashMap;

use super::repr::{Name, Program};
use relocation::{Relocation, RelocationKind};

pub fn encode(program: Program) -> Vec<u8> {
    let mut encoder = Encoder::default();
    encoder.encode_program(program);
    encoder.perform_relocations();
    encoder.code
}

#[derive(Debug, Default)]
struct Encoder {
    addresses: HashMap<Name, usize>,
    relocations: HashMap<Name, Vec<Relocation>>,
    code: Vec<u8>,
}
