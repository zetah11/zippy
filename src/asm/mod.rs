pub use alloc::Constraints;

mod alloc;
mod lower;

//use std::collections::HashMap;

use crate::resolve::names::Name;
use crate::{lir, mir};
//use alloc::regalloc;
use lower::lower;

pub fn asm(
    constraints: Constraints,
    types: &mir::Types,
    context: &mir::Context,
    entry: Option<Name>,
    decls: mir::Decls,
) -> lir::Program {
    let lowered = lower(entry, types, context, decls);

    /*
    let mut procs = HashMap::with_capacity(lowered.procs.len());
    for (name, proc) in lowered.procs {
        let proc = regalloc(&constraints, proc);
        procs.insert(name, proc);
    }

    lir_old::Program {
        procs,
        values: lowered.values,
    }
    */

    lowered
}
