use std::collections::{HashMap, HashSet};

use super::liveness::Position;
use crate::lir::Register;

type Interference = HashMap<Register, HashSet<Register>>;

pub fn priority(
    liveness: &HashMap<Register, HashSet<Position>>,
    interfere: &Interference,
) -> Vec<Register> {
    let mut scores: Vec<(Register, usize)> = Vec::new();

    for (reg, interferes) in interfere.iter() {
        let score = liveness
            .get(reg)
            .map(|positions| positions.len())
            .unwrap_or(0)
            .saturating_sub(interferes.len());

        insert_scored(&mut scores, *reg, score);
    }

    for (reg, positions) in liveness.iter() {
        let score = positions.len();
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

    let mut ndx = items.len().saturating_sub(1);
    while ndx > 0 {
        if items[ndx].1 > score {
            break;
        }

        ndx -= 1;
    }

    items.insert(ndx, (reg, score));
}
