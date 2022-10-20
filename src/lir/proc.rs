use std::collections::HashMap;

use super::{Block, BlockId, Branch, Instruction, Register};

#[derive(Debug)]
pub struct Procedure {
    pub param: Register,
    pub continuations: Vec<BlockId>,

    pub blocks: HashMap<BlockId, Block>,
    pub instructions: Vec<Instruction>,
    pub branches: Vec<Branch>,

    pub entry: BlockId,
    pub exits: Vec<BlockId>,
}

impl Procedure {
    pub fn get(&self, id: &BlockId) -> &Block {
        self.blocks.get(id).unwrap()
    }

    pub fn get_branch(&self, id: usize) -> &Branch {
        &self.branches[id]
    }

    pub fn get_instruction(&self, id: usize) -> &Instruction {
        &self.instructions[id]
    }

    pub fn has_block(&self, id: &BlockId) -> bool {
        self.blocks.contains_key(id)
    }
}

#[derive(Debug)]
pub struct ProcBuilder {
    param: Register,
    continuations: Vec<BlockId>,

    blocks: HashMap<BlockId, Block>,
    instructions: Vec<Instruction>,
    branches: Vec<Branch>,
    id: usize,
}

impl ProcBuilder {
    pub fn new(param: Register, continuations: impl IntoIterator<Item = BlockId>) -> Self {
        Self {
            param,
            continuations: continuations.into_iter().collect(),
            blocks: HashMap::new(),
            instructions: Vec::new(),
            branches: Vec::new(),
            id: 0,
        }
    }

    pub fn new_without_continuations(param: Register) -> Self {
        Self {
            param,
            continuations: Vec::new(),
            blocks: HashMap::new(),
            instructions: Vec::new(),
            branches: Vec::new(),
            id: 0,
        }
    }

    pub fn add(
        &mut self,
        id: BlockId,
        param: Option<Register>,
        instructions: impl IntoIterator<Item = Instruction>,
        branch: Branch,
    ) {
        let start_inst = self.instructions.len();
        self.instructions.extend(instructions);
        let end_inst = self.instructions.len();

        let insts = start_inst..end_inst;
        self.branches.push(branch);
        let branch = self.branches.len() - 1;

        assert!(self
            .blocks
            .insert(
                id,
                Block {
                    param,
                    insts,
                    branch
                }
            )
            .is_none());
    }

    pub fn add_continuations(&mut self, continuations: Vec<BlockId>) {
        assert!(self.continuations.is_empty());
        self.continuations = continuations;
    }

    pub fn fresh_id(&mut self) -> BlockId {
        let id = BlockId(self.id);
        self.id += 1;
        id
    }

    pub fn build(self, entry: BlockId, exits: Vec<BlockId>) -> Procedure {
        Procedure {
            param: self.param,
            continuations: self.continuations,
            blocks: self.blocks,
            instructions: self.instructions,
            branches: self.branches,
            entry,
            exits,
        }
    }
}
