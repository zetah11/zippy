use std::ops::Range;

use super::Register;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct BlockId(pub(crate) usize);

#[derive(Clone, Debug)]
pub struct Block {
    pub param: Vec<Register>,
    pub insts: Range<usize>,
    pub branch: usize,
}
