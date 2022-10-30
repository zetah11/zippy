//! Implements a simple "coherence check" to ensure no interfering registers have been assigned the same register.

use std::collections::{HashMap, HashSet};

use common::lir::{Register, Virtual};

use super::Allocator;

impl Allocator<'_> {
    pub fn check_consistency(&self, interference: &HashMap<Register, HashSet<Register>>) {
        for (reg, interferes) in interference.iter() {
            let this = match reg {
                Register::Virtual(Virtual { id, .. }) => self.mapping.get(id).unwrap(),
                _ => continue,
            };

            for other_reg in interferes.iter() {
                let other = match other_reg {
                    Register::Virtual(Virtual { id, .. }) => self.mapping.get(id).unwrap(),
                    reg => reg,
                };

                if this == other {
                    panic!("inconsistency: both {reg:?} and {other_reg:?} were allocated the same, but they interfere (on {this:?})");
                }
            }
        }
    }
}
