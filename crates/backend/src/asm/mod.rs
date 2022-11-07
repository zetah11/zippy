pub use alloc::{Constraints, RegisterInfo};

mod alloc;
mod lower;

use common::names::{Name, Names};
use common::{lir, mir};
use log::{info, trace};

use alloc::regalloc;
use lower::lower;

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
