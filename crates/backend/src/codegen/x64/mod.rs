mod block;
mod entry;
mod instruction;
mod pretty;
mod procedure;
mod values;

pub use self::pretty::pretty;
pub use self::values::CONSTRAINTS;

use std::collections::HashMap;
use std::fmt::{self, Display};

use common::lir;
use common::names::{Name, Names};
use iced_x86::code_asm::{CodeAssembler, CodeAssemblerResult, CodeLabel, IcedError};
use iced_x86::BlockEncoderOptions;
use target_lexicon::Triple;

use self::values::regid_to_reg;

pub fn encode(
    names: &mut Names,
    target: &Triple,
    entry: Option<Name>,
    program: lir::Program,
) -> Result<Encoded, Error> {
    let mut lowerer = Lowerer::new(names, entry, target, program)?;
    lowerer.lower_program()?;
    lowerer.assemble()
}

#[derive(Debug)]
pub struct Encoded {
    pub result: CodeAssemblerResult,
    pub labels: HashMap<Name, CodeLabel>,
}

#[derive(Debug)]
pub enum Error {
    UnsupportedTarget(Triple),
    IcedError(IcedError),
}

impl std::error::Error for Error {}
impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedTarget(triple) => write!(f, "the target '{triple}' is unsupported"),
            Self::IcedError(err) => write!(f, "iced error: '{err}'"),
        }
    }
}

impl From<IcedError> for Error {
    fn from(err: IcedError) -> Self {
        Self::IcedError(err)
    }
}

struct Lowerer<'a> {
    asm: CodeAssembler,
    names: &'a mut Names,

    program: lir::Program,

    labels: HashMap<Name, CodeLabel>,
    blocks: HashMap<lir::BlockId, CodeLabel>,
}

impl<'a> Lowerer<'a> {
    pub fn new(
        names: &'a mut Names,
        entry: Option<Name>,
        target: &Triple,
        program: lir::Program,
    ) -> Result<Self, Error> {
        let mut res = Self {
            asm: CodeAssembler::new(64).unwrap(),
            names,
            program,

            labels: HashMap::new(),
            blocks: HashMap::new(),
        };

        if let Some(entry) = entry {
            res.entry(entry, target)?;
        }

        Ok(res)
    }

    pub fn lower_program(&mut self) -> Result<(), Error> {
        let procs: Vec<_> = self.program.procs.drain().collect();

        for (name, procedure) in procs {
            self.lower_procedure(name, procedure)?;
        }

        Ok(())
    }

    pub fn assemble(mut self) -> Result<Encoded, Error> {
        let result = self.asm.assemble_options(
            0,
            BlockEncoderOptions::RETURN_RELOC_INFOS
                | BlockEncoderOptions::RETURN_CONSTANT_OFFSETS
                | BlockEncoderOptions::RETURN_NEW_INSTRUCTION_OFFSETS,
        )?;
        Ok(Encoded {
            result,
            labels: self.labels,
        })
    }

    fn label(&mut self, name: Name) -> CodeLabel {
        *self
            .labels
            .entry(name)
            .or_insert_with(|| self.asm.create_label())
    }

    fn block_label(&mut self, block: lir::BlockId) -> CodeLabel {
        *self
            .blocks
            .entry(block)
            .or_insert_with(|| self.asm.create_label())
    }

    fn set_label(&mut self, name: Name) -> Result<(), Error> {
        let label = self
            .labels
            .entry(name)
            .or_insert_with(|| self.asm.create_label());
        self.asm.set_label(label)?;
        Ok(())
    }

    fn set_block_label(&mut self, block: lir::BlockId) -> Result<(), Error> {
        let label = self
            .blocks
            .entry(block)
            .or_insert_with(|| self.asm.create_label());
        self.asm.set_label(label)?;
        Ok(())
    }
}
