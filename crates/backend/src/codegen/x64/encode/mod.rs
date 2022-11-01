mod block;
mod constant;
mod instruction;
mod procedure;
mod program;
mod register;
mod relocation;

use std::collections::HashMap;

pub use relocation::{Relocation, RelocationKind};

use common::names;

use super::repr::{Name, Program};

pub fn encode(mut program: Program) -> Encoded {
    // Encoding shouldn't need access to the names.
    let names = std::mem::take(&mut program.names);
    let mut encoder = Encoder::default();
    encoder.encode_program(program);
    //encoder.perform_relocations();

    let code_symbols = encoder
        .addresses
        .into_iter()
        .filter(|(name, _)| !names.is_block(name))
        .map(|(name, offset_size)| (*names.get(&name), offset_size))
        .collect();

    let data_symbols = encoder
        .constants
        .into_iter()
        .map(|(name, offset_size)| (*names.get(&name), offset_size))
        .collect();

    Encoded {
        code_symbols,
        data_symbols,
        code: encoder.code,
        data: encoder.data,
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
    pub data: Vec<u8>,
    pub code_symbols: Vec<(names::Name, (usize, usize))>,
    pub data_symbols: Vec<(names::Name, (usize, usize))>,
    pub relocations: Vec<(names::Name, Vec<Relocation>)>,
}

#[derive(Debug, Default)]
struct Encoder {
    addresses: HashMap<Name, (usize, usize)>,
    constants: HashMap<Name, (usize, usize)>,
    relocations: HashMap<Name, Vec<Relocation>>,
    code: Vec<u8>,
    data: Vec<u8>,
}
