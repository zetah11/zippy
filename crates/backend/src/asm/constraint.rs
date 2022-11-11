use common::lir::{CallingConvention, TypeId, Types};

pub type RegisterId = usize;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Place {
    /// Placed on the stack as an argument.
    Argument(usize),
    /// Placed as a local on the current stack.
    Local(usize),
}

#[derive(Clone, Debug)]
pub struct ProcedureAllocation {
    pub arguments: Vec<Place>,
    pub returns: Vec<Place>,
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
