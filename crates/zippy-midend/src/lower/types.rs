use zippy_common::mir::{Type, TypeId};
use zippy_common::thir::merge_insts;

use super::{HiType, Inst, Lowerer};

impl Lowerer<'_> {
    pub fn lower_type(&mut self, inst: &Inst, ty: HiType) -> TypeId {
        self.try_lower_type(inst, ty).unwrap()
    }

    pub fn try_lower_type(&mut self, inst: &Inst, ty: HiType) -> Option<TypeId> {
        match ty {
            HiType::Name(name) => match (inst.get(&name), self.named_types.get(&name)) {
                (Some(ty), None) => self.try_lower_type(inst, ty.clone()),
                (None, Some(ty)) => Some(*ty),
                _ => unreachable!(),
            },

            HiType::Instantiated(ty, other_inst) => {
                let inst = merge_insts(inst, &other_inst);
                self.try_lower_type(&inst, *ty)
            }

            HiType::Var(_mut, var) => {
                match self.subst.get(&var) {
                    Some((other_inst, ty)) => {
                        let inst = merge_insts(inst, other_inst);
                        self.try_lower_type(&inst, ty.clone())
                    }

                    // Assume an error has been produced if the var has no substitution.
                    None => Some(self.types.add(Type::Invalid)),
                }
            }

            HiType::Range(lo, hi) => Some(self.types.add(Type::Range(lo, hi))),

            HiType::Product(t, u) => {
                let t = self.try_lower_type(inst, *t)?;
                let u = self.try_lower_type(inst, *u)?;

                Some(self.types.add(Type::Product(vec![t, u])))
            }

            HiType::Fun(t, u) => {
                let t = self.try_lower_type(inst, *t)?;
                let u = self.try_lower_type(inst, *u)?;

                Some(self.types.add(Type::Fun(vec![t], vec![u])))
            }

            HiType::Type => unreachable!(),
            HiType::Number => Some(self.types.add(Type::Number)),
            HiType::Invalid => Some(self.types.add(Type::Invalid)),
        }
    }
}
