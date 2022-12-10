use zippy_common::thir::Type;

use super::Typer;

impl Typer<'_> {
    /// Infer the kind of a type.
    pub fn infer_type(&mut self, ty: &Type) -> Type {
        match ty {
            Type::Name(_) => todo!(),

            Type::Fun(t, u) | Type::Product(t, u) => {
                self.check_type(t, Type::Type);
                self.check_type(u, Type::Type);
                Type::Type
            }

            Type::Range(..) => Type::Type,
            Type::Number | Type::Invalid | Type::Type => Type::Type,

            Type::Var(..) => todo!(),
            Type::Instantiated(..) => todo!(),
        }
    }

    fn check_type(&mut self, _ty: &Type, _kind: Type) {
        todo!()
    }
}
