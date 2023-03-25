use zippy_common::source::Span;

use crate::check::constrained::DelayedConstraint;
use crate::messages::TypeMessages;

use super::{CoercionState, CoercionVar, Solver, Template, Type, UnifyVar};

impl Solver<'_> {
    /// Ensure `from` is either equal to or can be coerced to `into`.
    pub fn assign(&mut self, at: Span, id: CoercionVar, into: Type, from: Type) {
        self.flow(at, id, into, from);
    }

    /// Ensure the two given types are equal.
    pub fn equate(&mut self, at: Span, t: Type, u: Type) {
        self.unify(at, t, u);
    }

    /// Ensure `from` is either equal to or can be coerced to `into`, assuming
    /// beta-equivalence.
    fn beta_flow(&mut self, at: Span, id: CoercionVar, into: Template, from: Template) {
        let Template { ty: into } = into;
        let Template { ty: from } = from;

        self.flow(at, id, into, from);
    }

    /// Ensure that the two templates are beta-equivalent.
    fn beta_unify(&mut self, at: Span, t: Template, u: Template) {
        let Template { ty: t } = t;
        let Template { ty: u } = u;

        self.unify(at, t, u);
    }

    /// "Flow" the type of `from` to the type of `into`, ensuring that they are
    /// either equal or can be implicitly coerced.
    fn flow(&mut self, at: Span, id: CoercionVar, into: Type, from: Type) {
        match (into, from) {
            // If one of the types is an unsolved unification variable, unify
            // them. Otherwise, coerce.
            (into, Type::Var(var)) => match self.substitution.get(&var) {
                Some(from) => self.flow(at, id, into, from.clone()),
                None => {
                    self.coercions.mark(id, CoercionState::Equal);
                    self.unify(at, into, Type::Var(var));
                }
            },

            (Type::Var(var), from) => match self.substitution.get(&var) {
                Some(into) => self.flow(at, id, into.clone(), from),
                None => {
                    self.coercions.mark(id, CoercionState::Equal);
                    self.unify(at, Type::Var(var), from);
                }
            },

            // A trait can be coerced to a trait with fewer fields, assuming
            // each field is beta-coercible.
            (Type::Trait { values: into }, Type::Trait { values: mut from }) => {
                for (into_name, into_template) in into {
                    let raw = into_name.name(self.common_db());
                    let Some(&from_name) = from.keys().find(|name| name.name(self.common_db()) == raw) else {
                        let name = raw.text(self.common_db());
                        self.at(at).missing_field(name);
                        continue;
                    };

                    let from_template = from.remove(&from_name).unwrap();
                    self.beta_flow(at, id, into_template, from_template);
                }
            }

            // A smaller range can be coerced to a wider range.
            (Type::Range(big), Type::Range(small)) => {
                self.coercions.mark(id, CoercionState::Coercible);
                self.delayed.push(DelayedConstraint::Subset { big, small })
            }

            // An empty or unit range can be coerced to a unit type.
            (Type::Unit, Type::Range(range)) => {
                self.coercions.mark(id, CoercionState::Coercible);
                self.delayed.push(DelayedConstraint::UnitOrEmpty(range));
            }

            // Anything is equal to an invalid type.
            (Type::Invalid(_), _) | (_, Type::Invalid(_)) => {
                self.coercions.mark(id, CoercionState::Equal)
            }

            _ => {
                self.coercions.mark(id, CoercionState::Invalid);
                self.at(at).inequal_types();
            }
        }
    }

    /// Unify the two types, generating substitutions for any type variables
    /// such that the two types, when given that substitution, are equal.
    fn unify(&mut self, at: Span, t: Type, u: Type) {
        match (t, u) {
            (Type::Var(v), Type::Var(w)) if v == w => {}

            (t, Type::Var(v)) | (Type::Var(v), t) => {
                if let Some(u) = self.substitution.get(&v) {
                    return self.unify(at, t, u.clone());
                }

                if Self::occurs(&v, &t) {
                    self.at(at).recursive_type();
                } else {
                    self.substitution.insert(v, t);
                }
            }

            (Type::Trait { values: left }, Type::Trait { values: mut right }) => {
                for (left_name, left_template) in left {
                    let raw = left_name.name(self.common_db());
                    let Some(&right_name) = right.keys().find(|name| name.name(self.common_db()) == raw) else {
                        let name = raw.text(self.common_db());
                        self.at(at).missing_field(name);
                        continue;
                    };

                    let right_template = right.remove(&right_name).unwrap();
                    self.beta_unify(at, left_template, right_template);
                }

                if !right.is_empty() {
                    self.at(at).inequal_types();
                }
            }

            (Type::Range(first), Type::Range(second)) => self
                .delayed
                .push(DelayedConstraint::Equal { first, second }),

            (Type::Unit, Type::Range(range)) | (Type::Range(range), Type::Unit) => {
                self.delayed.push(DelayedConstraint::Unit(range))
            }

            (Type::Unit, Type::Unit) | (Type::Number, Type::Number) => {}

            (Type::Invalid(_), _) | (_, Type::Invalid(_)) => {}

            _ => {
                self.at(at).inequal_types();
            }
        }
    }

    /// Returns `true` if the given unification variable appears in the given
    /// type.
    fn occurs(v: &UnifyVar, t: &Type) -> bool {
        match t {
            Type::Trait { values } => values
                .values()
                .any(|template| Self::occurs(v, &template.ty)),

            Type::Var(w) => v == w,
            Type::Range { .. } | Type::Unit | Type::Number | Type::Invalid(_) => false,
        }
    }
}
