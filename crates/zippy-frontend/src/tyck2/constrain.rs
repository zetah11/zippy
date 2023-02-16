use zippy_common::{
    hir2::{Because, CoercionId, Constraint, Mutability, Type},
    message::Span,
};

use super::Typer;

impl Typer<'_> {
    /// Constrain two types `t` and `u` to be equal.
    pub fn equate(&mut self, _span: Span, _t: &Type, _u: &Type) {
        todo!()
    }

    /// Constrain the type `from` to be assignable to the type `into` and return
    /// an identifier for the necessary coercion.
    pub fn assign(&mut self, span: Span, into: &Type, from: &Type) -> CoercionId {
        let id = self.coercions.fresh();
        self.assign_at(span, id, into, from);
        id
    }

    pub fn assign_at(&mut self, _span: Span, _id: CoercionId, _into: &Type, _from: &Type) {
        todo!()
    }

    pub fn type_function(&mut self, because: Because, span: Span, ty: Type) -> (Type, Type) {
        match ty {
            Type::Fun(t, u) => (*t, *u),
            ty @ Type::Var(..) => {
                let t = Type::Var(Mutability::Mutable, self.context.fresh());
                let u = Type::Var(Mutability::Mutable, self.context.fresh());
                self.equate(
                    span,
                    &ty,
                    &Type::Fun(Box::new(t.clone()), Box::new(u.clone())),
                );

                (t, u)
            }

            _ => {
                // TODO: pretty-print type
                self.messages.at(span).tyck_not_a_fun(None::<&str>);
                (Type::Invalid, Type::Invalid)
            }
        }
    }

    pub fn type_number(&mut self, because: Because, span: Span, ty: Type) -> Type {
        match ty {
            ty @ Type::Range(..) => ty,
            Type::Number => Type::Number,
            Type::Invalid => Type::Invalid,

            ty @ Type::Var(..) => {
                self.constraints.push(Constraint::NumberType {
                    at: span,
                    because,
                    ty: ty.clone(),
                });

                ty
            }

            _ => {
                // TODO: pretty-print type
                self.messages.at(span).tyck_not_an_int(None::<&str>);
                Type::Invalid
            }
        }
    }

    pub fn type_tuple(&mut self, span: Span, ty: Type) -> (Type, Type) {
        match ty {
            Type::Product(t, u) => (*t, *u),
            Type::Invalid => (Type::Invalid, Type::Invalid),

            ty @ Type::Var(..) => {
                let a = Type::Var(Mutability::Mutable, self.context.fresh());
                let b = Type::Var(Mutability::Mutable, self.context.fresh());
                let other = Type::Product(Box::new(a.clone()), Box::new(b.clone()));
                self.equate(span, &ty, &other);
                (a, b)
            }

            _ => {
                // TODO: pretty-print type
                self.messages.at(span).tyck_tuple_type();
                (Type::Invalid, Type::Invalid)
            }
        }
    }
}
