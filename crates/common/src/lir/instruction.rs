use super::{BlockId, Register, Target, TypeId, Value};

#[derive(Clone, Debug)]
pub enum Instruction {
    Crash,

    Copy(Target, Value),
    Index(Target, Value, usize),
    Tuple(Target, Vec<Value>),
}

#[derive(Clone, Debug)]
pub enum Branch {
    Call(Value, Vec<(Register, TypeId)>, Vec<BlockId>),
    Return(BlockId, Vec<(Register, TypeId)>),
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
