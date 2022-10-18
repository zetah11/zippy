pub use block::{
    Block, BlockId, Branch, Cond, Global, Inst, Proc, Program, Register, Target, Value,
};
pub use build::ProcBuilder;

mod block;
mod build;
