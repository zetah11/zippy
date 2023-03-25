use zippy_common::invalid::Reason;
use zippy_common::names::RawName;
use zippy_common::source::Span;

use super::{Solver, Template, Type};
use crate::check::Constraint;
use crate::messages::{NameMessages, TypeMessages};
use crate::resolved::Alias;

impl Solver<'_> {
    /// Equate `ty` to be an instantiation of `template`.
    pub(super) fn instantiated(&mut self, at: Span, ty: Type, template: Template) {
        let Template { ty: actual } = template;
        self.equate(at, ty, actual);
    }

    /// Equate `ty` to be an instantiation of the template of `alias`.
    pub(super) fn instantiated_alias(&mut self, at: Span, ty: Type, alias: Alias) {
        let Some((_, template)) = self.aliases.get(&alias) else {
            self.constraints.push(Constraint::InstantiatedAlias(at, ty, alias));
            return;
        };

        let Template { ty: actual } = template.clone();
        self.equate(at, ty, actual);
    }

    /// Give `alias` the template of `name` in `of`.
    pub(super) fn alias(&mut self, at: Span, alias: Alias, of: Type, name: RawName) {
        match of {
            Type::Trait { values } => {
                for (item, template) in values {
                    if item.name(self.common_db()) == name {
                        assert!(self.aliases.insert(alias, (Some(item), template)).is_none());
                        return;
                    }
                }

                let name = name.text(self.common_db());
                self.at(at).unresolved_name(name);
                assert!(self
                    .aliases
                    .insert(
                        alias,
                        (
                            None,
                            Template {
                                ty: Type::Invalid(Reason::NameError),
                            }
                        )
                    )
                    .is_none());
            }

            Type::Invalid(reason) => {
                assert!(self
                    .aliases
                    .insert(
                        alias,
                        (
                            None,
                            Template {
                                ty: Type::Invalid(reason),
                            }
                        )
                    )
                    .is_none());
            }

            Type::Var(var) => match self.substitution.get(&var) {
                Some(of) => self.alias(at, alias, of.clone(), name),
                None => self.constraints.push(Constraint::Alias {
                    at,
                    alias,
                    of: Type::Var(var),
                    name,
                }),
            },

            _ => {
                self.at(at).not_a_trait();

                assert!(self
                    .aliases
                    .insert(
                        alias,
                        (
                            None,
                            Template {
                                ty: Type::Invalid(Reason::TypeError),
                            }
                        ),
                    )
                    .is_none());
            }
        }
    }
}
