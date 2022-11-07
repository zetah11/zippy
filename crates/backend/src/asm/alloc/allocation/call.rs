use common::lir::{Register, Type, TypeId};

use super::Allocator;

impl Allocator<'_> {
    /// Get the calling convention for the given `callee`. The first returned vector is for the call arguments, while the
    /// second is the for the return arguments (i.e. the parameters on the return continuation).
    pub fn calling_convention(&self, callee: &TypeId) -> (Vec<Register>, Vec<Register>) {
        let (args, rets) = match self.types.get(callee) {
            Type::Fun(args, rets) => (args, rets),
            _ => unreachable!(),
        };

        let arg_mapping = self.alloc_args(args);
        let ret_mapping = self.alloc_rets(rets);

        (arg_mapping, ret_mapping)
    }

    pub fn alloc_args(&self, args: &Vec<TypeId>) -> Vec<Register> {
        let mut arg_mapping = Vec::with_capacity(args.len());

        'args: for arg in args.iter() {
            let size = self.types.sizeof(arg);

            for physical in &self.constraints.parameters[arg_mapping.len()..] {
                // ouch
                let reg = &self
                    .constraints
                    .registers
                    .iter()
                    .find(|reg| &reg.id == physical)
                    .unwrap();

                if reg.size >= size {
                    arg_mapping.push(Register::Physical(*physical));
                    continue 'args;
                }
            }

            todo!()
        }

        assert!(args.len() == arg_mapping.len());
        arg_mapping
    }

    pub fn alloc_rets(&self, rets: &Vec<TypeId>) -> Vec<Register> {
        let mut ret_mapping = Vec::with_capacity(rets.len());

        'rets: for ret in rets.iter() {
            let size = self.types.sizeof(ret);

            for physical in &self.constraints.returns[ret_mapping.len()..] {
                let reg = &self
                    .constraints
                    .registers
                    .iter()
                    .find(|reg| &reg.id == physical)
                    .unwrap();

                if reg.size >= size {
                    ret_mapping.push(Register::Physical(*physical));
                    continue 'rets;
                }
            }

            todo!()
        }

        assert!(rets.len() == ret_mapping.len());
        ret_mapping
    }
}
