use std::collections::{HashMap, HashSet};
use std::ops::{Bound, RangeBounds};

use ranges::GenericRange;

use super::info::ProcInfo;
use super::live::Liveness;
use crate::lir::Register;

type Interference = HashMap<Register, HashSet<Register>>;

pub fn priority(info: &ProcInfo, liveness: &Liveness, interfere: &Interference) -> Vec<Register> {
    let mut scores: Vec<(Register, usize)> = Vec::new();

    for (reg, interferes) in interfere.iter() {
        let score = if info.args.contains(reg) {
            usize::MAX
        } else {
            live_length(liveness, reg).saturating_sub(interferes.len())
        };

        insert_scored(&mut scores, *reg, score);
    }

    for (reg, _) in liveness.regs.iter() {
        let score = live_length(liveness, reg);
        insert_scored(&mut scores, *reg, score);
    }

    scores.into_iter().map(|(reg, _)| reg).collect()
}

fn insert_scored(items: &mut Vec<(Register, usize)>, reg: Register, score: usize) {
    for (other, _) in items.iter() {
        if other == &reg {
            return;
        }
    }

    let mut ndx = items.len();
    while ndx > 0 {
        if items[ndx].1 > score {
            break;
        }

        ndx -= 1;
    }

    items.insert(ndx, (reg, score));
}

fn live_length(liveness: &Liveness, reg: &Register) -> usize {
    let ranges = match liveness.regs.get(reg) {
        Some(ranges) => ranges.as_slice(),
        None => return 0,
    };

    ranges.iter().map(range_length).sum()
}

fn range_length(range: &GenericRange<usize>) -> usize {
    let start = match range.start_bound() {
        Bound::Included(start) => *start,
        _ => unreachable!(),
    };

    let end = match range.end_bound() {
        Bound::Included(end) => *end,
        _ => unreachable!(),
    };

    // len [1, 1]  = len [1, 2) = 1
    end + 1 - start
}
