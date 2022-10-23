use std::collections::HashMap;

use super::block::Block;
use super::names::Name;

#[derive(Debug)]
pub struct Procedure {
    pub blocks: HashMap<Name, Block>,
    pub block_order: Vec<Name>,
}
