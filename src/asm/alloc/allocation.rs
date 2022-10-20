use std::collections::{HashMap, HashSet};

use super::interfere::interference;
use super::priority::prioritize;
use super::Constraints;
use crate::lir::{Procedure, Register, TypeId, Types};

type VirtualId = usize;

#[derive(Debug)]
pub struct Allocation {
    pub map: HashMap<VirtualId, Register>,
    pub frame_space: usize,
}

pub fn allocate(types: &Types, proc: &Procedure, constraints: &Constraints) -> Allocation {
    let (intf, info) = interference(proc);

    // Mapping from register to frame offset
    let mut frames: HashMap<VirtualId, (isize, TypeId)> = HashMap::new();

    // Mapping from register to physical register
    let mut mapping: HashMap<VirtualId, usize> = HashMap::new();

    let physical = (0..constraints.max_physical)
        .into_iter()
        .collect::<Vec<_>>();

    let mut max_frame_offset = 0;

    for reg in prioritize(&info, &intf) {
        let interferes: Vec<_> = intf.graph.get(&reg).into_iter().flatten().collect();
        let unavailable = interferes
            .iter()
            .flat_map(|reg| mapping.get(&reg.id))
            .copied()
            .collect::<HashSet<_>>();

        let mut available = physical
            .iter()
            .copied()
            .filter(|reg| !unavailable.contains(reg));

        match available.next() {
            Some(mapped) => assert!(mapping.insert(reg.id, mapped).is_none()),
            None => {
                // todo: unterrible this algorithm
                let mut off = 0;

                for reg in interferes.iter() {
                    if let Some((other_off, other_ty)) = frames.get(&reg.id) {
                        if *other_off < 0 {
                            continue;
                        }

                        off = off.max(*other_off as usize + types.sizeof(other_ty));
                    }
                }

                max_frame_offset = max_frame_offset.max(off + types.sizeof(&reg.ty));

                assert!(frames.insert(reg.id, (off as isize, reg.ty)).is_none());
            }
        }
    }

    let mut map = HashMap::new();

    for (reg, (frame_offset, ty)) in frames {
        assert!(frame_offset >= 0);
        assert!(map.insert(reg, Register::Frame(frame_offset, ty)).is_none());
    }

    for (reg, physical) in mapping {
        assert!(map.insert(reg, Register::Physical(physical)).is_none());
    }

    Allocation {
        map,
        frame_space: max_frame_offset as usize,
    }
}
