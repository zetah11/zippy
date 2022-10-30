mod allocation;
mod apply;
mod constraint;
mod info;
mod interfere;
mod liveness;
mod priority;

pub use constraint::{Constraints, RegisterInfo};

use std::collections::HashMap;

use common::lir::Program;

use allocation::allocate;
use apply::apply;

pub fn regalloc(constraints: &Constraints, program: Program) -> Program {
    let mut procs = HashMap::with_capacity(program.procs.len());

    for (name, procedure) in program.procs {
        let allocation = allocate(constraints, &program.types, &program.context, &procedure);
        let applied = apply(allocation, procedure);
        procs.insert(name, applied);
    }

    Program { procs, ..program }
}
