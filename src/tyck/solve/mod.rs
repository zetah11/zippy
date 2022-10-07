pub use unify::Unifier;

mod unify;

use super::{Type, Typer};
use crate::message::Span;

impl Typer {
    /// Check if `from` can be given where `into` is expected (i.e. if `from` is wider than `into`), and return the
    /// widest type.
    pub fn assignable(&mut self, span: Span, into: Type, from: Type) {
        self.unifier.unify(span, into, from);
    }

    pub fn contains_int(&mut self, (v, span): (i64, Span), ty: Type) -> Type {
        match ty {
            Type::Range(lo, hi) => {
                if !(lo <= v && v < hi) {
                    self.messages.at(span).tyck_outside_range(v, lo, hi);
                    Type::Invalid
                } else {
                    Type::Range(lo, hi)
                }
            }

            _ => Type::Invalid,
        }
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

    pub fn report_hole(&mut self, span: Span, ty: Type) -> Type {
        let var = self.context.fresh();
        self.assignable(span, Type::Var(var), ty);
        Type::Var(var)
    }
}
