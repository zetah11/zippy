pub use unify::Unifier;

mod unify;

use super::because::Because;
use super::constraint::Constraint;
use super::{Type, Typer};
use crate::message::Span;

impl Typer {
    /// Check if `from` can be given where `into` is expected (i.e. if `from` is wider than `into`), and return the
    /// widest type.
    pub fn assignable(&mut self, span: Span, into: Type, from: Type) {
        self.unifier.unify(span, into, from);
    }

    pub fn fun_type(&mut self, span: Span, ty: Type) -> (Type, Type) {
        match ty {
            Type::Fun(t1, t2) => (*t1, *t2),
            Type::Invalid => (Type::Invalid, Type::Invalid),
            ty => {
                self.messages
                    .at(span)
                    .tyck_not_a_fun(Some(format!("{ty:?}")));
                (Type::Invalid, Type::Invalid)
            }
        }
    }

    pub fn int_type(&mut self, span: Span, because: Because, ty: Type) -> Type {
        match ty {
            Type::Range(lo, hi) => Type::Range(lo, hi),
            Type::Invalid => Type::Invalid,

            Type::Var(v) => {
                if let Some(ty) = self.unifier.subst.get(&v) {
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
                    Type::Var(v)
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
        let expect = Type::Product(Box::new(Type::Var(t)), Box::new(Type::Var(u)));

        self.assignable(span, expect, ty);

        (Type::Var(t), Type::Var(u))
    }

    pub fn hole_type(&mut self, span: Span, ty: Type) -> Type {
        let var = self.context.fresh();
        self.assignable(span, Type::Var(var), ty);
        Type::Var(var)
    }
}
