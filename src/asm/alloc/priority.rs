use std::collections::{HashMap, HashSet};

use crate::lir::{Register, Virtual};

use super::info::ProcInfo;
use super::interfere::Interference;

pub fn prioritize(info: &ProcInfo, intf: &Interference) -> Vec<Virtual> {
    // prio = # uses - # interferences
    //     or infinity if the reg is returned or used in a call

    let mut uses: HashMap<Virtual, usize> = HashMap::new();
    for (_, used) in info.uses.iter() {
        for reg in used.iter() {
            if let Register::Virtual(reg) = reg {
                *uses.entry(*reg).or_insert(0) += 1;
            }
        }
    }

    let mut args: HashSet<Virtual> = HashSet::new();
    for arg in info.args.iter() {
        if let Register::Virtual(reg) = arg {
            args.insert(*reg);
        }
    }

    let mut res = vec![];
    let mut scores = vec![];

    for (reg, _) in intf.graph.iter() {
        if res.contains(reg) {
            continue;
        }

        let intfs = intf.graph.get(reg).map(|intfs| intfs.len()).unwrap_or(0);
        let uses = uses.get(reg).copied().unwrap_or(0);

        let score = if args.contains(reg) {
            usize::MAX
        } else {
            uses.saturating_sub(intfs)
        };

        let mut ndx = 0;
        while ndx < scores.len() {
            let left = scores[ndx];
            if left > score {
                break;
            }

            ndx += 1;
        }

        res.insert(ndx, *reg);
        scores.insert(ndx, score);
    }

    res
}
