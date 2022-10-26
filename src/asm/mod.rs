pub use alloc::{Constraints, RegisterInfo};

mod alloc;
mod lower;

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
    let prog = regalloc(&constraints, lowered);

    trace!("done lir generation");

    prog
}
