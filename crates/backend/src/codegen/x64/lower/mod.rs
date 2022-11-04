mod block;
mod entry;
mod globals;
mod instruction;
mod procedure;
mod value;

use std::collections::HashMap;

use common::lir;
use common::names::{Name, Names};
use target_lexicon::Triple;

use crate::codegen::CodegenError;

use super::repr as x64;

pub fn lower(
    names: &mut Names,
    target: &Triple,
    entry: Option<Name>,
    program: lir::Program,
) -> Result<x64::Program, CodegenError> {
    if let Some(entry) = entry {
        let mut lowerer = Lowerer::new(names, target, entry, program);
        lowerer.lower_program()?;

        Ok(x64::Program {
            procedures: lowerer.procedures,
            constants: lowerer.constants,
            names: lowerer.names,
        })
    } else {
        Ok(x64::Program {
            procedures: Vec::new(),
            constants: Vec::new(),
            names: x64::Names::new(),
        })
    }
}

#[derive(Debug)]
struct Lowerer<'a> {
    procedures: Vec<(x64::Name, x64::Procedure)>,
    constants: Vec<(x64::Name, Vec<u8>)>,
    names: x64::Names,
    blocks: HashMap<lir::BlockId, x64::Name>,
    entry: Name,

    target: &'a Triple,
    old_names: &'a mut Names,
    program: lir::Program,
}

impl<'a> Lowerer<'a> {
    pub fn new(
        names: &'a mut Names,
        target: &'a Triple,
        entry: Name,
        program: lir::Program,
    ) -> Self {
        Self {
            procedures: Vec::new(),
            constants: Vec::new(),
            names: x64::Names::new(),
            blocks: HashMap::new(),
            entry,

            target,
            old_names: names,
            program,
        }
    }

    pub fn lower_program(&mut self) -> Result<(), CodegenError> {
        self.lower_entry()?;
        let procs: Vec<_> = self.program.procs.drain().collect();
        let values: Vec<_> = self.program.values.drain().collect();

        for (name, value) in values {
            let name = self.lower_name(name);
            let value = self.lower_constant(name, value);
            self.constants.push((name, value));
        }

        for (name, proc) in procs {
            let name = self.lower_name(name);
            let proc = self.lower_procedure(name, proc);
            self.procedures.push((name, proc));
        }

        Ok(())
    }

    fn lower_name(&mut self, name: Name) -> x64::Name {
        self.names.add(name)
    }

    fn lower_block_id(&mut self, within: x64::Name, block: lir::BlockId) {
        let old = self.names.get(&within);
        let span = self.old_names.get_span(old);

        let new = self.old_names.fresh(span, *old);
        let new = self.names.add_block(new);

        self.blocks.insert(block, new);
    }
}
