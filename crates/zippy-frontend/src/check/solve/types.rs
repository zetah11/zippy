use zippy_common::messages::{Message, MessageMaker};
use zippy_common::names::RawName;
use zippy_common::source::Span;

use super::{Constraint, Solver, Template, Type};
use crate::messages::TypeMessages;

pub(super) enum NumericResult {
    Ok,
    Unsolved(Span, Type),
    Error(Vec<Message>),
}

impl Solver<'_> {
    /// Ensure the given type is a unit type (as in, it only has one value).
    pub(super) fn unitlike(&mut self, span: Span, ty: Type) {
        match ty {
            Type::Unit => {}
            Type::Range { .. } => todo!("ensure range is unit"),

            Type::Var(var) => match self.substitution.get(&var) {
                Some(ty) => self.unitlike(span, ty.clone()),
                None => self
                    .constraints
                    .push(Constraint::UnitLike(span, Type::Var(var))),
            },

            Type::Invalid(_) => {}

            _ => {
                self.at(span).not_unitlike();
            }
        }
    }

    /// Ensure the given type is numeric.
    pub(super) fn numeric(&self, span: Span, ty: Type) -> NumericResult {
        match ty {
            Type::Unit | Type::Number | Type::Range { .. } => NumericResult::Ok,

            Type::Var(var) => match self.substitution.get(&var) {
                Some(ty) => self.numeric(span, ty.clone()),
                None => NumericResult::Unsolved(span, Type::Var(var)),
            },

            Type::Invalid(_) => NumericResult::Ok,

            _ => {
                let mut messages = Vec::new();
                MessageMaker::new(&mut messages, span).not_numeric();
                NumericResult::Error(messages)
            }
        }
    }

    /// Ensure the given type is a string type.
    pub(super) fn textual(&mut self, span: Span, ty: Type) {
        match ty {
            Type::Unit => {}

            Type::Var(var) => match self.substitution.get(&var) {
                Some(ty) => self.textual(span, ty.clone()),
                None => self
                    .constraints
                    .push(Constraint::Textual(span, Type::Var(var))),
            },

            Type::Invalid(_) => {}

            _ => {
                self.at(span).not_textual();
            }
        }
    }

    /// Equate `target` with an instantiation of the type of the field `field`
    /// in the type `of`.
    ///
    /// For instance, if `of` is a type like `trait (fun id |T| (x: T) : T)` and
    /// `field` is `id`, then `target` would be equated with `?1 -> ?1` for some
    /// fresh type variable `?1`.
    pub(super) fn field(&mut self, at: Span, target: Type, of: Type, field: RawName) {
        match of {
            Type::Trait { values } => {
                for (value, template) in values {
                    if value.name(self.common_db()) == field {
                        let instantiated = self.instantiate(at, template);
                        self.equate(at, target, instantiated);
                        return;
                    }
                }

                let name = field.text(self.common_db());
                self.at(at).no_such_field(name);
            }

            Type::Var(var) => match self.substitution.get(&var) {
                Some(ty) => self.field(at, target, ty.clone(), field),
                None => self.constraints.push(Constraint::Field {
                    at,
                    target,
                    of: Type::Var(var),
                    field,
                }),
            },

            ty @ Type::Invalid(_) => self.equate(at, target, ty),

            _ => {
                self.at(at).not_a_trait();
            }
        }
    }

    /// Create a type equal to an instantiation of the given template.
    fn instantiate(&mut self, at: Span, template: Template) -> Type {
        let ty = self.fresh(at);
        self.constraints
            .push(Constraint::Instantiated(at, ty.clone(), template));
        ty
    }
}
