mod block;
mod instruction;
mod procedure;
mod program;
mod register;
mod relocation;

use std::collections::HashMap;

pub use relocation::{Relocation, RelocationKind};

use super::repr::{Name, Program};
use crate::resolve::names as resolve;

pub fn encode(mut program: Program) -> Encoded {
    // Encoding shouldn't need access to the names.
    let names = std::mem::take(&mut program.names);
    let mut encoder = Encoder::default();
    encoder.encode_program(program);
    //encoder.perform_relocations();

    let symbols = encoder
        .addresses
        .into_iter()
        .filter(|(name, _)| !names.is_block(name))
        .map(|(name, offset_size)| (*names.get(&name), offset_size))
        .collect();

    Encoded {
        symbols,
        code: encoder.code,
        relocations: encoder
            .relocations
            .into_iter()
            .map(|(name, relocations)| (*names.get(&name), relocations))
            .collect(),
    }
}

#[derive(Debug)]
pub struct Encoded {
    pub code: Vec<u8>,
    pub symbols: Vec<(resolve::Name, (usize, usize))>,
    pub relocations: Vec<(resolve::Name, Vec<Relocation>)>,
}

#[derive(Debug, Default)]
struct Encoder {
    addresses: HashMap<Name, (usize, usize)>,
    relocations: HashMap<Name, Vec<Relocation>>,
    code: Vec<u8>,
}
