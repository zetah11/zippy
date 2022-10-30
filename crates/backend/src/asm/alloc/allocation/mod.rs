mod branch;
mod call;
mod check;
mod fit;
mod unavailable;

use std::collections::HashMap;

use common::lir::{Context, Procedure, Register, Types, Virtual};

use super::constraint::Constraints;
use super::info::info;
use super::interfere::interference;
use super::liveness::liveness;
use super::priority::priority;

type VirtualId = usize;

#[derive(Debug)]
pub struct Allocation {
    pub mapping: HashMap<VirtualId, Register>,
    pub frame_space: usize,
}

pub fn allocate(
    constraints: &Constraints,
    types: &Types,
    context: &Context,
    procedure: &Procedure,
) -> Allocation {
    let allocator = Allocator::new(constraints, types, context);
    allocator.allocate(procedure)
}

struct Allocator<'a> {
    constraints: &'a Constraints,
    types: &'a Types,
    context: &'a Context,

    mapping: HashMap<usize, Register>,
    frame_space: usize,
}

impl<'a> Allocator<'a> {
    pub fn new(constraints: &'a Constraints, types: &'a Types, context: &'a Context) -> Self {
        Self {
            constraints,
            types,
            context,
            mapping: HashMap::new(),
            frame_space: 0,
        }
    }

    pub fn allocate(mut self, procedure: &Procedure) -> Allocation {
        let info = info(procedure);
        let liveness = liveness(&info, procedure);
        let interference = interference(&liveness);

        self.allocate_procedure(procedure);

        let priority = priority(&liveness, &interference);

        for reg in priority {
            let unavailable = self.unavailable_physical(&interference, &reg);

            let reg = match reg {
                Register::Virtual(reg) if !self.mapping.contains_key(&reg.id) => reg,
                _ => continue,
            };

            if let Some(physical) =
                self.first_fitting_reg(self.constraints.registers, unavailable, reg.ty)
            {
                assert!(self
                    .mapping
                    .insert(reg.id, Register::Physical(physical))
                    .is_none());
            } else {
                let unavailable = self.unavailable_frame(&interference, &Register::Virtual(reg));
                let offset = self.first_fitting_frame(unavailable, reg.ty);
                assert!(self
                    .mapping
                    .insert(reg.id, Register::Frame(offset, reg.ty))
                    .is_none());
                self.frame_space = self.frame_space.max(offset.max(0) as usize);
            }
        }

        self.mapping.shrink_to_fit();
        self.check_consistency(&interference);

        Allocation {
            mapping: self.mapping,
            frame_space: self.frame_space,
        }
    }

    fn allocate_procedure(&mut self, procedure: &Procedure) {
        let (param_names, param_types): (Vec<_>, Vec<_>) = procedure
            .params
            .iter()
            .map(|param| match param {
                Register::Virtual(Virtual { ty, id }) => (*id, *ty),
                _ => unreachable!(),
            })
            .unzip();

        let param_mapping = self.alloc_args(&param_types);

        for (param, mapped) in param_names.into_iter().zip(param_mapping) {
            assert!(self.mapping.insert(param, mapped).is_none());
        }

        for branch in procedure.branches.iter() {
            self.allocate_branch(procedure, branch);
        }
    }
}
