pub use unify::Unifier;

mod unify;

use common::message::Span;
use common::thir::{Because, Constraint};

use super::{Type, Typer};

impl Typer<'_> {
    /// Check if `from` can be given where `into` is expected (i.e. if `from` is wider than `into`), and return the
    /// widest type.
    pub fn assignable(&mut self, span: Span, into: Type, from: Type) {
        self.unifier.unify(span, into, from);
        let constraints: Vec<_> = self
            .unifier
            .worklist
            .drain(..)
            .map(|(at, into, from)| Constraint::Assignable { at, into, from })
            .collect();
        self.constraints.extend(constraints);
    }

    pub fn fun_type(&mut self, span: Span, ty: Type) -> (Type, Type) {
        let t = self.context.fresh();
        let u = self.context.fresh();
        let expect = Type::Fun(Box::new(Type::mutable(t)), Box::new(Type::mutable(u)));

        self.assignable(span, expect, ty);

        (Type::mutable(t), Type::mutable(u))
    }

    pub fn int_type(&mut self, span: Span, because: Because, ty: Type) -> Type {
        match ty {
            Type::Range(lo, hi) => Type::Range(lo, hi),
            Type::Invalid => Type::Invalid,

            Type::Var(mutable, v) => {
                if let Some((inst, ty)) = self.unifier.subst.get(&v) {
                    let because = if let Some(cause) = self.unifier.causes.get(&v) {
                        cause.clone()
                    } else {
                        because
                    };

                    self.int_type(span, because, ty.clone())
                } else {
                    self.constraints.push(Constraint::IntType {
                        at: span,
                        because,
                        ty,
                    });
                    Type::Var(mutable, v)
                }
            }

            ty => {
                self.messages
                    .at(span)
                    .tyck_not_an_int(Some(format!("{ty:?}")));
                Type::Invalid
            }
        }
    }

    pub fn tuple_type(&mut self, span: Span, ty: Type) -> (Type, Type) {
        let t = self.context.fresh();
        let u = self.context.fresh();
        let expect = Type::Product(Box::new(Type::mutable(t)), Box::new(Type::mutable(u)));

        self.assignable(span, expect, ty);

        (Type::mutable(t), Type::mutable(u))
    }

    pub fn hole_type(&mut self, span: Span, ty: Type) -> Type {
        let var = self.context.fresh();
        self.assignable(span, Type::mutable(var), ty);
        Type::mutable(var)
    }
}
