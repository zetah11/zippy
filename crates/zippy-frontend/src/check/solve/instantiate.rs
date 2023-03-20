use zippy_common::source::Span;

use super::{Solver, Template, Type};

impl Solver<'_> {
    /// Equate `ty` to be an instantiation of `template`.
    pub(super) fn instantiated(&mut self, at: Span, ty: Type, template: Template) {
        let Template { ty: actual } = template;
        self.equate(at, ty, actual);
    }
}
