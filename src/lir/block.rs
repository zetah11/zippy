use std::collections::HashMap;

use crate::resolve::names::Name;

#[derive(Debug)]
pub struct Proc {
    pub blocks: HashMap<BlockId, Block>,
    pub entry: BlockId,
    pub exit: BlockId,
}

impl Proc {
    pub fn get(&self, block: &BlockId) -> &Block {
        self.blocks.get(block).unwrap()
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct BlockId(pub(crate) usize);

#[derive(Debug)]
pub struct Block {
    pub insts: Vec<Inst>,
    pub branch: Branch,
}

#[derive(Debug)]
pub enum Branch {
    Return(Value),
    Jump(BlockId),
    JumpIf {
        conditional: Cond,
        left: Value,
        right: Value,
        consequence: BlockId,
        alternative: BlockId,
    },
}

#[derive(Debug)]
pub enum Inst {
    Move(Target, Value),

    Push(Value),
    Pop(Target),

    Call(Value),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Cond {
    Equal,
    Less,
    Greater,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Register {
    Virtual(usize),
    Physical(usize),
}

#[derive(Debug)]
pub enum Value {
    Register(Register),
    Location(Name),
    Integer(i64),
}

#[derive(Debug)]
pub enum Target {
    Register(Register),
    Location(Name),
}
