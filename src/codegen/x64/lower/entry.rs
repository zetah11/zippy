use std::collections::HashMap;

use super::super::repr::{Immediate, Instruction, Operand, Procedure, Register};
use super::Lowerer;
use crate::resolve::names::{Actual, Path};

impl Lowerer<'_> {
    pub fn lower_entry(&mut self) {
        let span = self.old_names.get_span(&self.entry);
        let start = self
            .old_names
            .add(span, Path(vec![], Actual::Lit("_start".into())));
        let start = self.names.add(start);
        let entry = self.names.add(self.entry);

        let insts = vec![
            Instruction::Call(Operand::Location(entry)),
            Instruction::Mov(
                Operand::Register(Register::Rax),
                Operand::Immediate(Immediate::Imm32(60)),
            ),
            Instruction::Syscall,
        ];

        self.procedures.insert(
            0,
            (
                start,
                Procedure {
                    prelude: insts,
                    block_order: vec![],
                    blocks: HashMap::new(),
                },
            ),
        )
    }
}
