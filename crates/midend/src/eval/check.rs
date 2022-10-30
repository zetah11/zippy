use common::message::Span;
use common::mir::pretty::Prettier;
use common::mir::{Type, TypeId};
use common::Driver;

use super::Lowerer;

impl<D: Driver> Lowerer<'_, D> {
    pub fn check_int_range(&mut self, span: Span, value: i64, ty: &TypeId) {
        match self.types.get(ty) {
            &Type::Range(lo, hi) => {
                if !(lo <= value && value < hi) {
                    let off_by_one = value == hi;
                    self.messages.at(span).elab_outside_range(
                        {
                            let prettier = Prettier::new(self.names, self.types);
                            prettier.pretty_type(ty)
                        },
                        off_by_one,
                    );
                }
            }

            Type::Invalid => {}

            _ => unreachable!(),
        }
    }
}
