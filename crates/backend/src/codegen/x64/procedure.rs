use common::lir::{BlockId, Branch, Procedure};
use common::names::Name;
use iced_x86::code_asm::{rbp, rsp};

use super::{Error, Lowerer};

impl Lowerer<'_> {
    pub fn lower_procedure(&mut self, name: Name, procedure: Procedure) -> Result<(), Error> {
        self.set_label(name)?;
        self.asm.push(rbp)?;
        self.asm.mov(rbp, rsp)?;

        if let Some(frame) = procedure.frame_space {
            if frame > 0 {
                self.asm.sub(rsp, i32::try_from(frame).unwrap())?;
            }
        }

        let mut blocks = self.block_order(&procedure);
        while !blocks.is_empty() {
            let block = blocks.remove(0);
            self.lower_block(&blocks[..], &procedure, block)?;
        }

        Ok(())
    }

    fn block_order(&self, procedure: &Procedure) -> Vec<BlockId> {
        let mut worklist = vec![procedure.entry];
        let mut blocks = Vec::with_capacity(procedure.blocks.len());

        while let Some(id) = worklist.pop() {
            worklist.extend(self.successor(procedure, &id).into_iter().rev());
            blocks.push(id);
        }

        // ?
        assert!(procedure.exits.iter().all(|id| blocks.contains(id)));

        blocks
    }

    fn successor(&self, procedure: &Procedure, block: &BlockId) -> Vec<BlockId> {
        let block = procedure.get(block);
        match procedure.get_branch(block.branch) {
            Branch::Call(_, _, conts) => {
                conts.clone() // assuming retcont is always first
            }

            Branch::Jump(to, _) => vec![*to],

            Branch::JumpIf {
                then: (then, _),
                elze: (elze, _),
                ..
            } => vec![*then, *elze],

            Branch::Return(..) => vec![],
        }
    }
}
