use super::{BlockId, Register, Target, Value};

#[derive(Clone, Debug)]
pub enum Instruction {
    Crash,

    Reserve(usize),

    Copy(Target, Value),
    Index(Target, Value, usize),
    Tuple(Target, Vec<Value>),
}

#[derive(Clone, Debug)]
pub enum Branch {
    Call(Value, Vec<Register>, Vec<BlockId>),
    Return(BlockId, Vec<Register>),
    Jump(BlockId, Vec<Value>),
    JumpIf {
        left: Value,
        cond: Condition,
        right: Value,
        then: (BlockId, Vec<Value>),
        elze: (BlockId, Vec<Value>),
    },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Condition {
    Less,
    Equal,
    Greater,
}
