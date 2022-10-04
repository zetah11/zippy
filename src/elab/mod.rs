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

mod lower;

use crate::mir;
use crate::tyck;
use crate::Driver;

pub fn elaborate(
    _driver: &mut impl Driver,
    decls: tyck::Decls<tyck::Type>,
) -> (mir::Types, mir::Decls) {
    lower::lower(decls)
}
