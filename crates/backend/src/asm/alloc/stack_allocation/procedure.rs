use common::lir::{Branch, Procedure, Register, ValueNode, Virtual};
use common::names::Name;

use super::super::info::info;
use super::super::interfere::interference;
use super::super::liveness::liveness;
use super::super::priority::priority;
use super::{AllocConstraints, Allocation, Allocator};
use crate::asm::{Place, ProcedureAllocation};

impl<Constraints: AllocConstraints> Allocator<'_, Constraints> {
    pub fn allocate_procedure(&mut self, name: &Name, procedure: &Procedure) -> Allocation {
        self.reset_info();

        let info = info(procedure);
        let liveness = liveness(&info, procedure);
        let interference = interference(&liveness);

        self.allocate_params(name, procedure);
        self.allocate_calls(procedure);

        for register in priority(&liveness, &interference) {
            let unavailable = self.unavailable_frame(&interference, &register);

            let Register::Virtual(register) = register else { continue; };
            if self.mapping.contains_key(&register.id) {
                continue;
            }

            let size = Constraints::sizeof(&self.program.types, &register.ty);
            let offset = self.first_fitting_frame(unavailable, size);

            self.frame_space = self.frame_space.max(offset.max(0) as usize + size);
            self.map(register.id, Place::Local(offset), size);
        }

        Allocation {
            mapping: self.mapping.drain().collect(),
            frame_space: self.frame_space,
        }
    }

    /// Allocate the arguments and return values of all the function calls in the procedure.
    fn allocate_calls(&mut self, procedure: &Procedure) {
        for branch in procedure.branches.iter() {
            let Branch::Call(func, args, conts) = branch else { continue; };

            let name = match func.node {
                ValueNode::Name(name) => name,
                _ => todo!(),
            };

            let convention = self.convention(&name).as_call();
            let args = args.iter().map(|(reg, _)| *reg).collect();
            let rets = conts
                .first() // return continuation is always first
                .map(|id| procedure.get(id).params.clone())
                .into_iter()
                .collect();

            self.allocate_with_convention(convention, args, rets);
        }
    }

    /// Allocate the parameters and return values in the procedure.
    fn allocate_params(&mut self, name: &Name, procedure: &Procedure) {
        let convention = self.convention(name).clone();
        let params = procedure.params.to_vec();
        let rets = procedure
            .branches
            .iter()
            .filter_map(|branch| match branch {
                Branch::Return(_, values) => Some(values.iter().map(|(reg, _)| *reg).collect()),
                _ => None,
            })
            .collect();

        self.allocate_with_convention(convention, params, rets);
    }

    fn allocate_with_convention(
        &mut self,
        convention: ProcedureAllocation,
        args: Vec<Register>,
        rets: Vec<Vec<Register>>,
    ) {
        for (param, place) in args.into_iter().zip(convention.arguments) {
            let Register::Virtual(Virtual { id, ty }) = param else { unreachable!() };
            let size = Constraints::sizeof(&self.program.types, &ty);

            self.map(id, place, size);
        }

        for rets in rets {
            for (param, place) in rets.into_iter().zip(convention.returns.iter()) {
                let Register::Virtual(Virtual { id, ty }) = param else { unreachable!() };
                let size = Constraints::sizeof(&self.program.types, &ty);

                self.map(id, *place, size);
            }
        }
    }

    fn reset_info(&mut self) {
        self.mapping.clear();
        self.frame_space = 0;
    }
}
