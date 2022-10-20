use std::collections::HashMap;

pub use block::{Block, BlockId};
pub use instruction::{Branch, Condition, Instruction};
pub use proc::{ProcBuilder, Procedure};
pub use register::{Register, Virtual};
pub use types::{Type, TypeId, Types};
pub use value::{Target, Value};

mod block;
mod instruction;
mod proc;
mod register;
mod types;
mod value;

use crate::resolve::names::Name;

#[derive(Debug)]
pub struct Program {
    pub procs: HashMap<Name, Procedure>,
    pub values: HashMap<Name, Global>,
    pub types: Types,
}

#[derive(Debug)]
pub struct Global {
    pub data: i64,
}
