use super::Evaluator;
use crate::message::Span;
use crate::mir::{Type, TypeId};
use crate::Driver;

impl<'a, D: Driver> Evaluator<'a, D> {
    pub fn check_int(&mut self, span: Span, value: i64, ty: &TypeId) {
        match self.types.get(ty) {
            Type::Range(lo, hi) => {
                if !(lo <= &value && &value < hi) {
                    self.messages.at(span).elab_outside_range(*lo, *hi);
                }
            }

            Type::Invalid => {}

            // This should not happen, right?
            _ => unreachable!(),
        }
    }
}
