use std::collections::{HashMap, HashSet};

use common::lir::Register;

use super::{AllocConstraints, Allocator, Place};

impl<C: AllocConstraints> Allocator<'_, C> {
    pub fn unavailable_frame(
        &self,
        interference: &HashMap<Register, HashSet<Register>>,
        reg: &Register,
    ) -> Vec<(Place, usize)> {
        let interferes = match interference.get(reg) {
            Some(interferes) => interferes,
            None => return vec![],
        };

        interferes
            .iter()
            .copied()
            .filter_map(|reg| match reg {
                Register::Virtual(id) => self.mapping.get(&id.id).copied(),
                _ => None,
            })
            .collect()
    }
}
