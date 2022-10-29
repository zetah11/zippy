use super::Allocator;
use crate::lir::{Branch, Procedure, Register, Value, Virtual};

impl Allocator<'_> {
    /// Allocate the parameters to a `call` or `return` for the given branch.
    pub fn allocate_branch(&mut self, procedure: &Procedure, branch: &Branch) {
        match branch {
            Branch::Call(fun, args, conts) => {
                let fun_ty = match fun {
                    Value::Register(Register::Virtual(Virtual { ty, .. })) => *ty,
                    Value::Name(name) => self.context.get(name),

                    Value::Register(_) | Value::Integer(_) => unreachable!(),
                };

                let args: Vec<_> = args
                    .iter()
                    .map(|arg| match arg {
                        Register::Virtual(Virtual { id, .. }) => *id,
                        _ => unreachable!(),
                    })
                    .collect();

                let (arg_regs, ret_regs) = self.calling_convention(&fun_ty);

                assert!(arg_regs.len() == args.len());
                for (arg, reg) in args.into_iter().zip(arg_regs) {
                    assert!(self.mapping.insert(arg, reg).is_none());
                }

                if let Some(retc) = conts.iter().next() {
                    let block = procedure.get(retc);
                    assert!(ret_regs.len() == block.params.len());

                    for (param, reg) in block.params.iter().zip(ret_regs) {
                        match param {
                            Register::Virtual(Virtual { id, .. }) => {
                                assert!(self.mapping.insert(*id, reg).is_none())
                            }
                            _ => unreachable!(),
                        }
                    }
                }
            }

            Branch::Return(_, values) => {
                let (ret_names, ret_types): (Vec<_>, Vec<_>) = values
                    .iter()
                    .map(|value| match value {
                        Register::Virtual(Virtual { ty, id }) => (*id, *ty),
                        _ => unreachable!(),
                    })
                    .unzip();

                let ret_mapping = self.alloc_rets(&ret_types);

                for (ret, reg) in ret_names.into_iter().zip(ret_mapping) {
                    assert!(self.mapping.insert(ret, reg).is_none());
                }
            }

            _ => {}
        };
    }
}
