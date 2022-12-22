use zippy_common::message::Span;
use zippy_common::thir::Type;

use super::Typer;

impl Typer<'_> {
    /// Infer the kind of a type.
    pub fn infer_type(&mut self, at: Span, ty: &Type) -> Type {
        match ty {
            Type::Name(_) => todo!(),

            Type::Fun(t, u) | Type::Product(t, u) => {
                self.check_type(at, t, Type::Type);
                self.check_type(at, u, Type::Type);
                Type::Type
            }

            Type::Range(..) => Type::Type,
            Type::Number | Type::Invalid | Type::Type => Type::Type,

            Type::Var(..) => Type::mutable(self.context.fresh()),
            Type::Instantiated(..) => todo!(),
        }
    }

    pub fn check_type(&mut self, at: Span, ty: &Type, kind: Type) {
        let inferred = self.infer_type(at, ty);
        // no coercion should happen for kinds
        let _ = self.assignable(at, kind, inferred);
    }
}
