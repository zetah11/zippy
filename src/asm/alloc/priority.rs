use std::collections::HashMap;

use crate::lir::{Register, Virtual};

use super::info::ProcInfo;
use super::interfere::Interference;

pub fn prioritize(info: &ProcInfo, intf: &Interference) -> Vec<Virtual> {
    // prio = # uses - # interferences

    let mut uses: HashMap<Virtual, usize> = HashMap::new();
    for (_, used) in info.uses.iter() {
        for reg in used.iter() {
            if let Register::Virtual { reg, ndx: _ndx } = reg {
                *uses.entry(*reg).or_insert(0) += 1;
            }
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

        let score = uses - intfs;

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
