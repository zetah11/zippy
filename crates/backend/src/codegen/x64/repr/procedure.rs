use std::collections::HashMap;

use super::block::Block;
use super::instruction::Instruction;
use super::names::Name;

#[derive(Debug)]
pub struct Procedure {
    pub prelude: Vec<Instruction>,
    pub block_order: Vec<Name>,
    pub blocks: HashMap<Name, Block>,
}
