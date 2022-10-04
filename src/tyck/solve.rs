use super::{Type, Typer};
use crate::message::Span;

impl Typer {
    /// Check if `from` can be given where `into` is expected (i.e. if `from` is wider than `into`), and return the
    /// widest type.
    pub fn assignable(&mut self, span: Span, into: Type, from: Type) -> Type {
        match (into, from) {
            (Type::Invalid, _) | (_, Type::Invalid) => Type::Invalid,
            (Type::Number, Type::Range(..)) => Type::Number,

            (Type::Range(lo1, hi1), Type::Range(lo2, hi2)) => {
                if !(lo1 <= lo2 && hi1 >= hi2) {
                    self.messages
                        .at(span)
                        .tyck_narrow_range((lo1, hi1), (lo2, hi2));
                }

                Type::Range(lo1, hi1)
            }

            (Type::Fun(t1, u1), Type::Fun(t2, u2)) => {
                self.assignable(span, *t2, (*t1).clone());
                let u1 = self.assignable(span, *u1, *u2);
                Type::Fun(t1, Box::new(u1))
            }

            (expected, actual) => {
                self.messages
                    .at(span)
                    .tyck_incompatible(Some(format!("{expected:?}")), Some(format!("{actual:?}")));
                Type::Invalid
            }
        }
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
}
