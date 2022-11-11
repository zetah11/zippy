use common::lir::{CallingConvention, Type, TypeId, Types};
use common::sizes::min_range_size;

use crate::asm::{AllocConstraints, Place, ProcedureAllocation};

pub struct Constraints;

impl AllocConstraints for Constraints {
    const NAME: &'static str = "x86_64";

    fn sizeof(types: &Types, ty: &TypeId) -> usize {
        match types.get(ty) {
            Type::Range(lo, hi) => {
                let min = min_range_size(*lo, *hi) + usize::from(*lo < 0 || *hi < 0);
                if min > 8 {
                    todo!()
                }

                8
            }

            Type::Product(ts) => ts.iter().map(|ty| Self::sizeof(types, ty)).sum(),

            Type::Fun(..) => 8,
        }
    }

    fn offsetof(types: &Types, ty: &TypeId, ndx: usize) -> usize {
        match types.get(ty) {
            Type::Product(ties) => {
                assert!(ties.len() > ndx);
                ties.iter()
                    .take(ndx)
                    .map(|ty| Self::sizeof(types, ty))
                    .sum()
            }

            Type::Fun(..) | Type::Range(..) => unreachable!(),
        }
    }

    fn convention(
        types: &Types,
        convention: CallingConvention,
        args: &[TypeId],
        rets: &[TypeId],
    ) -> Option<ProcedureAllocation> {
        match convention {
            CallingConvention::Corollary => Some(convention_corollary(types, args, rets)),
            _ => None,
        }
    }
}

/// `corollary` calling convention. Arguments are pushed left-to-right on the stack. Return values are placed on the
/// stack in space reserved by the callee.
fn convention_corollary(types: &Types, args: &[TypeId], rets: &[TypeId]) -> ProcedureAllocation {
    let mut offset = 0;
    let mut returns = Vec::with_capacity(rets.len());

    for ret in rets.iter() {
        returns.push(Place::Argument(offset));
        offset += Constraints::sizeof(types, ret);
    }

    let mut arguments = Vec::with_capacity(args.len());

    for arg in args.iter().rev() {
        arguments.push(Place::Argument(offset));
        offset += Constraints::sizeof(types, arg);
    }

    ProcedureAllocation { arguments, returns }
}
