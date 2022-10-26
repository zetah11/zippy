mod approximate;
mod precise;
mod range;

pub use precise::Liveness;

use self::approximate::approximate_liveness;
use self::precise::precise_liveness;
use super::constraint::Constraints;
use super::info::ProcInfo;
use crate::lir::Procedure;

pub fn liveness(constraints: &Constraints, proc: &Procedure) -> (Liveness, ProcInfo) {
    let (approx, info) = approximate_liveness(proc);
    let precise = precise_liveness(&approx, proc, constraints);
    (precise, info)
}
