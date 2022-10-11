use super::Lowerer;
use crate::message::Span;
use crate::mir::{Type, TypeId};
use crate::Driver;

impl<D: Driver> Lowerer<'_, D> {
    pub fn check_int_range(&mut self, span: Span, value: i64, ty: &TypeId) {
        match self.types.get(ty) {
            &Type::Range(lo, hi) => {
                println!("{lo:?} {hi:?} {value:?}");
                if !(lo <= value && value < hi) {
                    self.messages.at(span).elab_outside_range(lo, hi);
                }
            }

            Type::Invalid => {}

            _ => unreachable!(),
        }
    }
}
