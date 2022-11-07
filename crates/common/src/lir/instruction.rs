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
    Jump(BlockId, Vec<Register>),
    JumpIf {
        left: Value,
        cond: Condition,
        right: Value,
        args: Vec<Register>,
        then: BlockId,
        elze: BlockId,
    },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Condition {
    Less,
    Equal,
    Greater,
}
