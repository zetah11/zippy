use std::collections::HashMap;

use super::super::repr::{Immediate, Instruction, Operand, Procedure, Register};
use super::super::Target;
use super::Lowerer;
use crate::resolve::names::{Actual, Path};

impl Lowerer<'_> {
    pub fn lower_entry(&mut self) {
        match self.target {
            Target::Linux64 => self.lower_entry_linux(),
            Target::Windows64 => self.lower_entry_windows(),
        }
    }

    fn lower_entry_linux(&mut self) {
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

    fn lower_entry_windows(&mut self) {
        let span = self.old_names.get_span(&self.entry);
        let main = self
            .old_names
            .add(span, Path(vec![], Actual::Lit("_WinMain".into())));

        let exit_process = self
            .old_names
            .add(span, Path(vec![], Actual::Lit("ExitProcess".into())));

        let main = self.names.add(main);
        let exit_process = self.names.add(exit_process);
        let entry = self.names.add(self.entry);

        let insts = vec![
            Instruction::Call(Operand::Location(entry)),
            Instruction::Mov(
                Operand::Register(Register::Rcx),
                Operand::Register(Register::Rdi),
            ),
            Instruction::Call(Operand::Location(exit_process)),
        ];

        self.procedures.insert(
            0,
            (
                main,
                Procedure {
                    prelude: insts,
                    block_order: vec![],
                    blocks: HashMap::new(),
                },
            ),
        )
    }
}
