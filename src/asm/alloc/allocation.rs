use std::collections::{HashMap, HashSet};

use super::interfere::interference;
use super::priority::prioritize;
use crate::lir::{Proc, Register};

#[derive(Debug)]
pub struct Allocation {
    pub map: HashMap<Register, Register>,
    pub frame_space: usize,
}

#[derive(Debug)]
pub struct Constraints {
    pub max_physical: usize,
}

pub fn allocate(proc: &Proc, constraints: &Constraints) -> Allocation {
    let (intf, info) = interference(proc);

    // Mapping from register to frame offset
    let mut frames: HashMap<Register, isize> = HashMap::new();

    // Mapping from register to physical register
    let mut mapping: HashMap<Register, usize> = HashMap::new();

    let physical = (0..constraints.max_physical)
        .into_iter()
        .collect::<Vec<_>>();

    let mut max_frame_offset = 0;

    for reg in prioritize(&info, &intf) {
        let interferes: Vec<_> = intf.graph.get(&reg).into_iter().flatten().collect();
        let unavailable = interferes
            .iter()
            .flat_map(|reg| mapping.get(reg))
            .copied()
            .collect::<HashSet<_>>();

        let mut available = physical
            .iter()
            .copied()
            .filter(|reg| !unavailable.contains(reg));

        match available.next() {
            Some(mapped) => assert!(mapping.insert(reg, mapped).is_none()),
            None => {
                let unavailable = interferes
                    .iter()
                    .flat_map(|reg| frames.get(reg))
                    .copied()
                    .collect::<HashSet<_>>();

                let mut off = 0;
                while unavailable.contains(&off) {
                    off += 1;
                }

                max_frame_offset = max_frame_offset.max(off + 1);

                assert!(frames.insert(reg, off).is_none());
            }
        }
    }

    let mut map = HashMap::new();

    for (reg, frame_offset) in frames {
        assert!(frame_offset >= 0);
        assert!(map.insert(reg, Register::Frame(frame_offset)).is_none());
    }

    for (reg, physical) in mapping {
        assert!(map.insert(reg, Register::Physical(physical)).is_none());
    }

    Allocation {
        map,
        frame_space: max_frame_offset as usize,
    }
}
