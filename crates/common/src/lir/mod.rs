use std::collections::HashMap;

pub use block::{Block, BlockId};
pub use clobber::clobbered;
pub use info::{Info, NameInfo};
pub use instruction::{Branch, Condition, Instruction};
pub use proc::{ProcBuilder, Procedure};
pub use register::{Register, Virtual};
pub use types::{Context, Type, TypeId, Types};
pub use value::{Target, TargetNode, Value, ValueNode};

mod block;
mod clobber;
mod info;
mod instruction;
mod proc;
mod register;
mod types;
mod value;

use crate::names::Name;

#[derive(Debug)]
pub struct Program {
    pub procs: HashMap<Name, Procedure>,
    pub values: HashMap<Name, Value>,
    pub types: Types,
    pub context: Context,
    pub info: NameInfo,
}
