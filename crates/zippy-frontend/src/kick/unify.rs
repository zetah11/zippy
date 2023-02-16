use zippy_common::message::Span;

use super::kinds::{Kind, UniVar};
use super::pretty::Prettier;
use super::Kinder;

impl Kinder<'_> {
    pub fn unify(&mut self, span: Span, a: Kind, b: Kind) {
        match (a, b) {
            (Kind::Var(var), Kind::Var(war)) if var == war => {}

            (Kind::Var(var), b) => {
                if let Some(a) = self.subst.get(&var) {
                    return self.unify(span, a.clone(), b);
                }

                if Self::occurs(var, &b) {
                    self.messages.at(span).kick_recursive_kind();
                    self.subst.insert(var, Kind::Invalid);
                } else {
                    self.subst.insert(var, b);
                }
            }

            (a, Kind::Var(war)) => {
                if let Some(b) = self.subst.get(&war) {
                    return self.unify(span, a, b.clone());
                }

                if Self::occurs(war, &a) {
                    self.messages.at(span).kick_recursive_kind();
                    self.subst.insert(war, Kind::Invalid);
                } else {
                    self.subst.insert(war, a);
                }
            }

            (Kind::Invalid, _) | (_, Kind::Invalid) => {}
            (Kind::Type, Kind::Type) => {}

            (Kind::Function(a, b), Kind::Function(c, d))
            | (Kind::Product(a, b), Kind::Product(c, d)) => {
                self.unify(span, *a, *c);
                self.unify(span, *b, *d);
            }

            (expected, actual) => {
                let prettier = Prettier::new(&self.subst);

                let expected = prettier.pretty(&mut self.namer, &expected);
                let actual = prettier.pretty(&mut self.namer, &actual);

                self.messages
                    .at(span)
                    .kick_incompatible_kinds(expected, actual);
            }
        }
    }

    fn occurs(var: UniVar, kind: &Kind) -> bool {
        match kind {
            Kind::Type | Kind::Invalid => false,

            Kind::Function(a, b) | Kind::Product(a, b) => {
                Self::occurs(var, a) || Self::occurs(var, b)
            }

            Kind::Var(war) => var == *war,
        }
    }
}
