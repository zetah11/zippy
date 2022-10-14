use std::collections::{HashMap, HashSet};

use super::info::ProcInfo;
use super::live::liveness;
use crate::lir::{Proc, Register};

pub fn interference(proc: &Proc) -> (Interference, ProcInfo) {
    let (live, info) = liveness(proc);
    let mut graph: HashMap<Register, HashSet<Register>> = HashMap::new();

    for (_, regs) in live.live_in.into_iter().chain(live.live_out) {
        let regs: im::HashSet<_> = regs.into();

        for reg in regs.iter().copied() {
            let interfere = regs.without(&reg);
            graph.entry(reg).or_default().extend(interfere);
        }
    }

    (Interference { graph }, info)
}

#[derive(Debug)]
pub struct Interference {
    pub graph: HashMap<Register, HashSet<Register>>,
}
