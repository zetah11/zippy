//! Elaboration is responsible for converting type checked code into a lower-level form without any kind of implicit
//! types.
//!
//! Effectively, the elaboration "pass" consists of two steps:
//!
//! 1. Type checking/inference, where type annotations are checked for validity and other types are inferred
//! 2. Partial evaluation, where pure and type-level expressions are evaluated
//!
//! Finally, the resulting code is converted into the [mid-level intermediate representation](crate::mir), on which the
//! compiler can begin lowering code for generation.

pub mod eval;
mod lower;

use log::{info, trace};

use crate::mir;
use crate::resolve::names::Names;
use crate::tyck::TypeckResult;
use crate::Driver;

pub fn elaborate(
    driver: &mut impl Driver,
    names: &mut Names,
    tyckres: TypeckResult,
) -> (mir::Types, mir::Decls) {
    info!("beginning elaboration");

    let (types, res) = lower::lower(driver, &tyckres.subst, tyckres.decls);
    let res = eval::evaluate(driver, names, &types, res);

    trace!("done elaborating");

    (types, res)
}
