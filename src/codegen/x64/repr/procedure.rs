use std::collections::HashMap;

use super::block::Block;
use super::names::Name;

#[derive(Debug)]
pub struct Procedure {
    blocks: HashMap<Name, Block>,
}
