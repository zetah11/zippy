use std::collections::{HashMap, HashSet};

use super::live::Liveness;
use crate::lir::Register;

pub fn interference(liveness: &Liveness) -> HashMap<Register, HashSet<Register>> {
    let mut res: HashMap<Register, HashSet<Register>> = HashMap::new();

    for (reg, range) in liveness.regs.iter() {
        for (other, other_range) in liveness.regs.iter() {
            if other == reg {
                continue;
            }

            let intersection = range.clone().intersect(other_range.clone());
            if !intersection.is_empty() {
                res.entry(*reg).or_default().insert(*other);
            }
        }
    }

    res
}
