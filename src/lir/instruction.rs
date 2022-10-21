use super::{BlockId, Target, Value};

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
    Call(Value, Value, Vec<BlockId>),
    Return(BlockId, Value),
    Jump(BlockId, Option<Value>),
    JumpIf {
        left: Value,
        cond: Condition,
        right: Value,
        then: (BlockId, Option<Value>),
        elze: (BlockId, Option<Value>),
    },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Condition {
    Less,
    Equal,
    Greater,
}
