use zippy_common::source::Span;

use super::instanced::{Instance, InstanceVar, Type};
use super::Clarifier;
use crate::messages::ClarifyMessages;

impl Clarifier {
    pub fn unify(&mut self, at: Span, left: Type, right: Type) {
        match (left, right) {
            (Type::Trait { instance: i }, Type::Trait { instance: j }) => {
                self.equate_instances(at, i, j);
            }

            (Type::Number | Type::Range(_), Type::Number | Type::Range(_)) => {}
            (Type::Invalid(_), _) | (_, Type::Invalid(_)) => {}

            _ => unreachable!("the type checker should not produce inequal types"),
        }
    }

    pub fn equate_instances(&mut self, at: Span, left: Instance, right: Instance) {
        match (left, right) {
            (Instance::Concrete(i), Instance::Concrete(j)) if i == j => {}

            (Instance::Parameter(_), _) | (_, Instance::Parameter(_)) => {
                unreachable!("parameters should have been instantiatied")
            }

            (Instance::Var(v), i) | (i, Instance::Var(v)) => {
                if let Some(v) = self.substitution.get(&v) {
                    self.equate_instances(at, *v, i);
                } else if !self.occurs(&v, &i) {
                    self.substitution.insert(v, i);
                } else {
                    self.at(at).recursive_instance();
                }
            }

            _ => self.at(at).incompatible_instances(),
        }
    }

    fn occurs(&mut self, var: &InstanceVar, instance: &Instance) -> bool {
        match instance {
            Instance::Concrete(_) => false,
            Instance::Var(v) => v == var,
            Instance::Parameter(_) => unreachable!(),
        }
    }
}
