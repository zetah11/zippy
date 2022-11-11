use common::lir::{CallingConvention, TypeId, Types};

pub type RegisterId = usize;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Place {
    /// Placed on the stack as an argument.
    Argument(usize),

    /// Placed on the stack as a parameter.
    Parameter(usize),

    /// Placed on the stack as a local.
    Local(usize),
}

#[derive(Clone, Debug)]
pub struct ProcedureAllocation {
    pub arguments: Vec<Place>,
    pub returns: Vec<Place>,
}

impl ProcedureAllocation {
    pub fn as_call(&self) -> Self {
        let arguments = self
            .arguments
            .iter()
            .map(|arg| match arg {
                Place::Local(offset) => Place::Local(*offset),
                Place::Parameter(offset) => Place::Argument(*offset),
                Place::Argument(_) => {
                    unreachable!("not a ProcedureAllocation for a function signature")
                }
            })
            .collect();

        let returns = self
            .returns
            .iter()
            .map(|ret| match ret {
                Place::Local(offset) => Place::Local(*offset),
                Place::Parameter(offset) => Place::Argument(*offset),
                Place::Argument(_) => {
                    unreachable!("not a ProcedureAllocation for a function signature")
                }
            })
            .collect();

        Self { arguments, returns }
    }
}

pub trait AllocConstraints {
    const NAME: &'static str;

    /// Get the expected placement of arguments and return values given the procedure calling convention, the argument
    /// types, and the return types. Returns `None` if the convention is not supported.
    fn convention(
        types: &Types,
        convention: CallingConvention,
        args: &[TypeId],
        rets: &[TypeId],
    ) -> Option<ProcedureAllocation>;

    fn sizeof(types: &Types, ty: &TypeId) -> usize;

    fn offsetof(types: &Types, ty: &TypeId, at: usize) -> usize;
}
