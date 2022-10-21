pub use alloc::{Constraints, RegisterInfo};

mod alloc;
mod lower;

use std::collections::HashMap;

use log::{info, trace};

use crate::resolve::names::Name;
use crate::{lir, mir};
use alloc::regalloc;
use lower::lower;

pub fn asm(
    constraints: Constraints,
    types: &mir::Types,
    context: &mir::Context,
    entry: Option<Name>,
    decls: mir::Decls,
) -> lir::Program {
    info!("beginning lir generation");

    let lowered = lower(entry, types, context, decls);

    let mut procs = HashMap::with_capacity(lowered.procs.len());
    for (name, proc) in lowered.procs {
        let proc = regalloc(&constraints, &lowered.types, proc);
        procs.insert(name, proc);
    }

    trace!("done lir generation");

    lir::Program {
        procs,
        values: lowered.values,
        types: lowered.types,
    }
}
