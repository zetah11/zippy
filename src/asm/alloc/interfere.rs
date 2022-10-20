use std::collections::{HashMap, HashSet};

use super::info::ProcInfo;
use super::live::liveness;
use crate::lir::{Procedure, Register, Virtual};

pub fn interference(proc: &Procedure) -> (Interference, ProcInfo) {
    let (live, info) = liveness(proc);
    let mut graph: HashMap<Virtual, HashSet<Virtual>> = HashMap::new();

    for (_, regs) in live.live_in.into_iter().chain(live.live_out) {
        let regs: im::HashSet<_> = regs
            .into_iter()
            .flat_map(|reg| match reg {
                Register::Virtual { reg, ndx: _ndx } => Some(reg),
                _ => None,
            })
            .collect();

        for reg in regs.iter().copied() {
            let interfere = regs.without(&reg);
            graph.entry(reg).or_default().extend(interfere);
        }
    }

    (Interference { graph }, info)
}

#[derive(Debug)]
pub struct Interference {
    pub graph: HashMap<Virtual, HashSet<Virtual>>,
}
