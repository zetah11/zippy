use std::ops::{Bound, RangeBounds};

use ranges::{GenericRange, Ranges};

use crate::lir::{BlockId, Procedure};

pub type LiveRange = GenericRange<usize>;
pub type LiveRanges = Ranges<usize>;

pub fn range(start: usize, end: usize) -> LiveRange {
    GenericRange::new_closed(start, end)
}

pub fn point(at: usize) -> LiveRange {
    GenericRange::singleton(at)
}

pub fn span(a: LiveRange, b: LiveRange) -> LiveRange {
    let start = match (a.start_bound(), b.start_bound()) {
        (Bound::Included(a), Bound::Included(b)) => (*a).min(*b),
        _ => unreachable!(),
    };

    let end = match (a.end_bound(), b.end_bound()) {
        (Bound::Included(a), Bound::Included(b)) => (*a).max(*b),
        _ => unreachable!(),
    };

    range(start, end)
}

pub fn branch_range(proc: &Procedure, branch: usize) -> LiveRange {
    point(proc.instructions.len() + branch)
}

pub fn block_header_range(proc: &Procedure, block: BlockId) -> LiveRange {
    point(proc.instructions.len() + proc.branches.len() + block.0)
}

pub fn procedure_header_range(proc: &Procedure) -> LiveRange {
    point(proc.instructions.len() + proc.branches.len() + proc.blocks.len())
}
