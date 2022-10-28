use std::collections::HashMap;

use crate::message::{Messages, Span};
use crate::tyck::because::Because;
use crate::tyck::{Type, UniVar};

#[derive(Debug, Default)]
pub struct Unifier {
    pub subst: HashMap<UniVar, Type>,
    pub causes: HashMap<UniVar, Because>,
    pub messages: Messages,
}

impl Unifier {
    pub fn new() -> Self {
        Self {
            subst: HashMap::new(),
            causes: HashMap::new(),
            messages: Messages::new(),
        }
    }

    pub fn unify(&mut self, span: Span, expected: Type, actual: Type) {
        match (expected, actual) {
            (Type::Range(lo1, hi1), Type::Range(lo2, hi2)) => {
                if !(lo1 <= lo2 && hi1 >= hi2) {
                    self.messages
                        .at(span)
                        .tyck_narrow_range((lo1, hi1), (lo2, hi2));
                }
            }

            (Type::Number, Type::Range(..)) => {}

            (Type::Fun(t1, u1), Type::Fun(t2, u2)) => {
                self.unify(span, *t1, *t2);
                self.unify(span, *u1, *u2);
            }

            (Type::Product(t1, u1), Type::Product(t2, u2)) => {
                self.unify(span, *t1, *t2);
                self.unify(span, *u1, *u2);
            }

            (Type::Var(v), Type::Var(w)) if v == w => {}

            (Type::Var(v), u) => {
                if let Some(t) = self.subst.get(&v).cloned() {
                    self.unify(span, t, u);
                } else if Self::occurs(&v, &u) {
                    self.messages
                        .at(span)
                        .tyck_recursive_inference(format!("{v:?}"), format!("{u:?}"));
                    self.set(v, Type::Invalid);
                } else {
                    self.set(v, u);
                    self.causes.insert(v, Because::Unified(span));
                }
            }

            (t, Type::Var(w)) => {
                if let Some(u) = self.subst.get(&w).cloned() {
                    self.unify(span, t, u);
                } else if Self::occurs(&w, &t) {
                    self.messages
                        .at(span)
                        .tyck_recursive_inference(format!("{w:?}"), format!("{t:?}"));
                    self.set(w, Type::Invalid);
                } else {
                    self.set(w, t);
                    self.causes.insert(w, Because::Unified(span));
                }
            }

            (Type::Invalid, _) | (_, Type::Invalid) => {}

            (expected, actual) => {
                self.messages
                    .at(span)
                    .tyck_incompatible(Some(format!("{expected:?}")), Some(format!("{actual:?}")));
            }
        }
    }

    fn occurs(var: &UniVar, ty: &Type) -> bool {
        match ty {
            Type::Invalid | Type::Number => false,
            Type::Range(_, _) => false,
            Type::Fun(t, u) => Self::occurs(var, t) || Self::occurs(var, u),
            Type::Product(t, u) => Self::occurs(var, t) || Self::occurs(var, u),
            Type::Var(war) => var == war,
        }
    }

    fn set(&mut self, var: UniVar, ty: Type) {
        assert!(self.subst.insert(var, ty).is_none());
    }
}
