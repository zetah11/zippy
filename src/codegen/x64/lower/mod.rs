mod block;
mod entry;
mod instruction;
mod procedure;
mod value;

use std::collections::HashMap;

use super::repr as x64;
use super::Target;
use crate::lir;
use crate::resolve::names::{Name, Names};

pub fn lower(
    names: &mut Names,
    target: &Target,
    entry: Option<Name>,
    program: lir::Program,
) -> x64::Program {
    if let Some(entry) = entry {
        let mut lowerer = Lowerer::new(names, target, entry, program);
        lowerer.lower_program();

        x64::Program {
            procedures: lowerer.procedures,
            names: lowerer.names,
        }
    } else {
        x64::Program {
            procedures: Vec::new(),
            names: x64::Names::new(),
        }
    }
}

#[derive(Debug)]
struct Lowerer<'a> {
    procedures: Vec<(x64::Name, x64::Procedure)>,
    names: x64::Names,
    blocks: HashMap<lir::BlockId, x64::Name>,
    entry: Name,

    target: &'a Target,
    old_names: &'a mut Names,
    program: lir::Program,
}

impl<'a> Lowerer<'a> {
    pub fn new(
        names: &'a mut Names,
        target: &'a Target,
        entry: Name,
        program: lir::Program,
    ) -> Self {
        Self {
            procedures: Vec::new(),
            names: x64::Names::new(),
            blocks: HashMap::new(),
            entry,

            target,
            old_names: names,
            program,
        }
    }

    pub fn lower_program(&mut self) {
        self.lower_entry();
        let procs: Vec<_> = self.program.procs.drain().collect();
        for (name, proc) in procs {
            let name = self.lower_name(name);
            let proc = self.lower_procedure(name, proc);
            self.procedures.push((name, proc));
        }
    }

    fn lower_name(&mut self, name: Name) -> x64::Name {
        self.names.add(name)
    }

    fn lower_block_id(&mut self, within: x64::Name, block: lir::BlockId) {
        let old = self.names.get(&within);
        let span = self.old_names.get_span(old);

        let new = self.old_names.fresh(span, Some(*old));
        let new = self.names.add_block(new);

        self.blocks.insert(block, new);
    }
}
