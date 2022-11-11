use common::lir::{Info, Type};
use common::names::{Actual, Name, Path};
use iced_x86::code_asm::{rax, rbp, rcx, rdi, rsp};
use target_lexicon::{Architecture, BinaryFormat, Environment, OperatingSystem, Triple};

use super::{Constraints, Error, Lowerer, RelocationKind};
use crate::asm::{AllocConstraints, Place, ProcedureAllocation};

impl Lowerer<'_> {
    pub fn entry(&mut self, entry: Name, target: &Triple) -> Result<(), Error> {
        assert!(target.architecture == Architecture::X86_64);

        match target {
            Triple {
                operating_system: OperatingSystem::Windows,
                binary_format: BinaryFormat::Coff | BinaryFormat::Unknown,
                environment: Environment::Msvc | Environment::Unknown,
                ..
            } => self.entry_windows(entry),

            Triple {
                operating_system: OperatingSystem::Linux,
                binary_format: BinaryFormat::Elf | BinaryFormat::Unknown,
                ..
            } => self.entry_linux(entry),

            target => return Err(Error::UnsupportedTarget(target.clone())),
        }

        Ok(())
    }

    fn entry_linux(&mut self, entry: Name) {
        let span = self.names.get_span(&entry);
        let start = self
            .names
            .add(span, Path(None, Actual::Lit("_start".into())));

        let convention = self.entry_convention(&entry);
        let entry = self.label(entry);

        let return_space = self.calling_return_space(&convention);
        let return_place = convention.returns.first();

        self.set_label(start);
        self.asm.mov(rbp, rsp).unwrap();
        self.asm.sub(rsp, return_space as i32).unwrap();
        self.asm.call(entry).unwrap();

        match return_place {
            Some(Place::Argument(offset)) => self.asm.mov(rdi, rsp + *offset).unwrap(),
            Some(Place::Local(_)) => todo!(),
            None => self.asm.xor(rdi, rdi).unwrap(),

            Some(Place::Parameter(_)) => unreachable!(),
        }

        self.asm.mov(rax, 60i64).unwrap();
        self.asm.syscall().unwrap();
    }

    fn entry_windows(&mut self, entry: Name) {
        let span = self.names.get_span(&entry);

        let main = self
            .names
            .add(span, Path(None, Actual::Lit("wmain".into())));

        let exit_process = self
            .names
            .add(span, Path(None, Actual::Lit("ExitProcess".into())));

        let uint = self.program.types.add(Type::Range(0, 4294967295));
        let ep_sig = self.program.types.add(Type::Fun(vec![uint], vec![]));

        self.program
            .info
            .add(exit_process, Info::EXTERN | Info::PROCEDURE);
        self.program.context.add(exit_process, ep_sig);

        let convention = self.entry_convention(&entry);
        let entry = self.label(entry);

        let return_space = self.calling_return_space(&convention);
        let return_place = convention.returns.first();

        self.set_label(main);
        self.asm.mov(rbp, rsp).unwrap();
        self.asm.sub(rsp, return_space as i32).unwrap();
        self.asm.call(entry).unwrap();

        match return_place {
            Some(Place::Argument(offset)) => self.asm.mov(rcx, rsp + *offset).unwrap(),
            Some(Place::Local(_)) => todo!(),
            None => self.asm.xor(rcx, rcx).unwrap(),

            Some(Place::Parameter(_)) => unreachable!(),
        }

        let exit_process = self.relocation_here(exit_process, RelocationKind::RelativeNext);
        self.asm.call(exit_process).unwrap();
    }

    fn entry_convention(&self, entry: &Name) -> ProcedureAllocation {
        let (args, rets) = {
            let ty = self.program.context.get(entry);
            match self.program.types.get(&ty) {
                Type::Fun(args, rets) => (args, rets),
                _ => unreachable!(),
            }
        };

        let convention = self.program.info.get_convention(entry).unwrap_or_default();

        Constraints::convention(&self.program.types, convention, args, rets)
            .unwrap()
            .as_call()
    }
}
