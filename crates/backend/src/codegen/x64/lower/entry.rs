use std::collections::HashMap;

use common::names::{Actual, Path};
use target_lexicon::{Architecture, BinaryFormat, Environment, OperatingSystem, Triple};

use super::super::repr::{Immediate, Instruction, Operand, Procedure, Register};
use super::Lowerer;
use crate::codegen::CodegenError;

impl Lowerer<'_> {
    pub fn lower_entry(&mut self) -> Result<(), CodegenError> {
        match self.target {
            Triple {
                architecture: Architecture::X86_64,
                operating_system: OperatingSystem::Windows,
                binary_format: BinaryFormat::Coff | BinaryFormat::Unknown,
                environment: Environment::Msvc | Environment::Unknown,
                vendor: _,
            } => self.lower_entry_windows(),

            Triple {
                architecture: Architecture::X86_64,
                operating_system: OperatingSystem::Linux,
                binary_format: BinaryFormat::Elf | BinaryFormat::Unknown,
                environment: _,
                vendor: _,
            } => self.lower_entry_linux(),

            target => return Err(CodegenError::TargetNotSupported(target.clone())),
        }

        Ok(())
    }

    fn lower_entry_linux(&mut self) {
        let span = self.old_names.get_span(&self.entry);
        let start = self
            .old_names
            .add(span, Path(None, Actual::Lit("_start".into())));
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
            .add(span, Path(None, Actual::Lit("_WinMain".into())));

        let exit_process = self
            .old_names
            .add(span, Path(None, Actual::Lit("ExitProcess".into())));

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
