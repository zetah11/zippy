mod alloc;
mod constraint;
mod lower;

pub use constraint::{AllocConstraints, Place, ProcedureAllocation, RegisterId};

use common::names::{Name, Names};
use common::{lir, mir, Driver};
use log::{info, trace};

use alloc::allocate;
use lower::lower;

pub fn asm<Constraints: AllocConstraints>(
    driver: &mut impl Driver,
    types: &mir::Types,
    context: &mir::Context,
    names: &Names,
    entry: Option<Name>,
    decls: mir::Decls,
) -> lir::Program {
    info!("beginning lir generation");

    let lowered = lower::<Constraints>(entry, types, context, names, decls);
    let program = allocate::<Constraints>(driver, names, lowered);

    trace!("done lir generation");

    program
}

/*
pub fn asm(
    constraints: Constraints,
    types: &mir::Types,
    context: &mir::Context,
    names: &Names,
    entry: Option<Name>,
    decls: mir::Decls,
) -> lir::Program {
    info!("beginning lir generation");

    let lowered = lower(entry, types, context, names, decls);
    let prog = regalloc(&constraints, lowered);

    trace!("done lir generation");

    prog
}
*/
