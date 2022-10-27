use std::collections::{HashMap, HashSet};

use super::constraint::Constraints;
use super::info::info;
use super::interfere::interference;
use super::liveness::liveness;
use super::priority::priority;
use super::RegisterInfo;
use crate::lir::{Procedure, Register, TypeId, Types};

type VirtualId = usize;

#[derive(Debug)]
pub struct Allocation {
    pub mapping: HashMap<VirtualId, Register>,
    pub frame_space: usize,
}

pub fn allocate(types: &Types, constraints: &Constraints, procedure: &Procedure) -> Allocation {
    let info = info(procedure);
    let liveness = liveness(&info, procedure);
    let interference = interference(&liveness);

    println!("{liveness:?}\n{interference:?}");

    let priority = priority(&info, &liveness, &interference);

    let mut mapping = HashMap::with_capacity(priority.len());
    let mut frame_space = 0;

    let parameters: Vec<_> = constraints
        .parameters
        .iter()
        .map(|id| constraints.registers[*id])
        .collect();

    for arg in procedure.params.iter() {
        let unavailable = unavailable_physical(&mapping, &interference, arg);
        let reg = match arg {
            Register::Virtual(reg) => reg,
            _ => continue,
        };

        if let Some(physical) = first_fitting_reg(types, &parameters, unavailable, reg.ty) {
            assert!(mapping
                .insert(reg.id, Register::Physical(physical))
                .is_none());
        } else {
            todo!()
        }
    }

    for reg in priority {
        let unavailable = unavailable_physical(&mapping, &interference, &reg);

        let reg = match reg {
            Register::Virtual(reg) if !mapping.contains_key(&reg.id) => reg,
            _ => continue,
        };

        if let Some(physical) = first_fitting_reg(types, constraints.registers, unavailable, reg.ty)
        {
            assert!(mapping
                .insert(reg.id, Register::Physical(physical))
                .is_none());
        } else {
            let unavailable = unavailable_frame(&mapping, &interference, &Register::Virtual(reg));
            let offset = first_fitting_frame(types, unavailable, reg.ty);
            assert!(mapping
                .insert(reg.id, Register::Frame(offset, reg.ty))
                .is_none());
            frame_space = frame_space.max(offset.max(0) as usize);
        }
    }

    mapping.shrink_to_fit();

    Allocation {
        mapping,
        frame_space,
    }
}

fn first_fitting_frame(types: &Types, mut unavailable: Vec<(isize, TypeId)>, ty: TypeId) -> isize {
    unavailable.sort_by(|(off1, _), (off2, _)| off1.cmp(off2));
    let size = isize::try_from(types.sizeof(&ty)).unwrap();

    let mut off = match unavailable.get(0) {
        Some((off, ty)) => {
            if *off < 0 || size < *off {
                return 0;
            } else {
                off + isize::try_from(types.sizeof(ty)).unwrap()
            }
        }
        None => 0,
    };

    for i in 0..(unavailable.len().saturating_sub(1)) {
        let bottom = unavailable[i].0 + isize::try_from(types.sizeof(&unavailable[i].1)).unwrap();
        let top = unavailable[i + 1].0;
        let gap = top - bottom;
        assert!(gap > 0);

        if gap >= size {
            off = bottom;
            break;
        } else if top >= 0 {
            off = top + isize::try_from(types.sizeof(&unavailable[i + 1].1)).unwrap();
        }
    }

    off
}

fn first_fitting_reg(
    types: &Types,
    possible: &[RegisterInfo],
    unavailable: Vec<usize>,
    ty: TypeId,
) -> Option<usize> {
    let size = types.sizeof(&ty);
    for reg in possible.iter() {
        if reg.size >= size && !unavailable.contains(&reg.id) {
            return Some(reg.id);
        }
    }

    None
}

fn unavailable_frame(
    mapping: &HashMap<usize, Register>,
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
            Register::Virtual(id) => match mapping.get(&id.id) {
                Some(Register::Frame(size, ty)) => Some((*size, *ty)),
                _ => None,
            },
            Register::Frame(size, ty) => Some((size, ty)),
            _ => None,
        })
        .collect()
}

fn unavailable_physical(
    mapping: &HashMap<usize, Register>,
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
            Register::Virtual(id) => match mapping.get(&id.id) {
                Some(Register::Physical(id)) => Some(*id),
                _ => None,
            },
            Register::Physical(id) => Some(id),
            _ => None,
        })
        .collect()
}
