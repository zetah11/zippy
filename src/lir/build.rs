use std::collections::HashMap;

use super::{Block, BlockId, Proc};

#[derive(Debug, Default)]
pub struct ProcBuilder {
    blocks: HashMap<BlockId, Block>,
    curr: usize,
}

impl ProcBuilder {
    pub fn new() -> Self {
        Self {
            blocks: HashMap::new(),
            curr: 0,
        }
    }

    pub fn add(&mut self, block: Block) -> BlockId {
        let id = BlockId(self.curr);
        self.curr += 1;
        self.blocks.insert(id, block);
        id
    }

    pub fn build(self, entry: BlockId, exit: BlockId) -> Proc {
        Proc {
            blocks: self.blocks,
            entry,
            exit,
        }
    }
}
