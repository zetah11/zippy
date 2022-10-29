use std::collections::{HashMap, HashSet};

use super::Allocator;
use crate::lir::{Register, TypeId};

impl Allocator<'_> {
    pub fn unavailable_frame(
        &self,
        interference: &HashMap<Register, HashSet<Register>>,
        reg: &Register,
    ) -> Vec<(isize, TypeId)> {
        let interferes = match interference.get(reg) {
            Some(interferes) => interferes,
            None => return vec![],
        };

        interferes
            .iter()
            .copied()
            .filter_map(|reg| match reg {
                Register::Virtual(id) => match self.mapping.get(&id.id) {
                    Some(Register::Frame(size, ty)) => Some((*size, *ty)),
                    _ => None,
                },
                Register::Frame(size, ty) => Some((size, ty)),
                _ => None,
            })
            .collect()
    }

    pub fn unavailable_physical(
        &self,
        interference: &HashMap<Register, HashSet<Register>>,
        reg: &Register,
    ) -> Vec<usize> {
        let interferes = match interference.get(reg) {
            Some(interferes) => interferes,
            None => return vec![],
        };

        interferes
            .iter()
            .copied()
            .filter_map(|reg| match reg {
                Register::Virtual(id) => match self.mapping.get(&id.id) {
                    Some(Register::Physical(id)) => Some(*id),
                    _ => None,
                },
                Register::Physical(id) => Some(id),
                _ => None,
            })
            .collect()
    }
}
