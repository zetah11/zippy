use std::collections::{HashMap, HashSet};

use super::liveness::Position;
use crate::lir::Register;

pub fn interference(
    liveness: &HashMap<Register, HashSet<Position>>,
) -> HashMap<Register, HashSet<Register>> {
    let mut res: HashMap<Register, HashSet<Register>> = HashMap::new();

    for (reg, positions) in liveness.iter() {
        for (other, other_positions) in liveness.iter() {
            if other == reg {
                continue;
            }

            if overlapping(positions, other_positions) {
                res.entry(*reg).or_default().insert(*other);
            }
        }
    }

    res
}

fn overlapping(a: &HashSet<Position>, b: &HashSet<Position>) -> bool {
    a.iter().any(|pos| b.contains(pos))
}
