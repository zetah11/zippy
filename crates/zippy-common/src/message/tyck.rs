use super::{Diagnostic, Label, MessageAdder};
use crate::Number;

const AMBIGUOUS: &str = "ET05";
const INCOMPATIBLE_TYPES: &str = "ET00";
const INSTANTIATE_NOT_GENERIC: &str = "ET09";
const INSTANTIATE_NON_NAME: &str = "ET07";
const INSTANTIATE_WRONG_ARITY: &str = "ET08";
const NARROW_RANGE: &str = "ET01";
const NO_PROGRESS: &str = "ET02";
const NOT_A_FUN: &str = "ET03";
const NOT_AN_INT: &str = "ET04";
const RECURSIVE: &str = "ET06";
const TUPLE_TYPE: &str = "ET10";

impl<'a> MessageAdder<'a> {
    pub fn tyck_ambiguous(&mut self) {
        let labels = vec![Label::primary(self.at)
            .with_message("this expression potentially has multiple valid types")];

        self.add(
            Diagnostic::error()
                .with_code(AMBIGUOUS)
                .with_message("cannot infer the type of an ambiguous expression")
                .with_labels(labels),
        );
    }

    pub fn tyck_incompatible(
        &mut self,
        expected: Option<impl Into<String>>,
        actual: Option<impl Into<String>>,
    ) {
        let labels =
            match (expected, actual) {
                (Some(expected), Some(actual)) => vec![Label::primary(self.at).with_message(
                    format!("expected '{}', got '{}'", expected.into(), actual.into()),
                )],
                _ => Vec::new(),
            };

        self.add(
            Diagnostic::error()
                .with_code(INCOMPATIBLE_TYPES)
                .with_message("incompatible types")
                .with_labels(labels),
        );
    }

    pub fn tyck_instantiate_not_generic(&mut self, name: Option<impl Into<String>>) {
        let labels = if let Some(name) = name {
            vec![Label::primary(self.at)
                .with_message(format!("'{}' is not a polymorphic type", name.into()))]
        } else {
            vec![Label::primary(self.at)]
        };

        self.add(
            Diagnostic::error()
                .with_code(INSTANTIATE_NOT_GENERIC)
                .with_message("cannot explicitly instantiate non-generic value")
                .with_labels(labels),
        );
    }

    pub fn tyck_instantiate_non_name(&mut self) {
        let labels = vec![Label::primary(self.at)];

        self.add(
            Diagnostic::error()
                .with_code(INSTANTIATE_NON_NAME)
                .with_message("only named values can be explicitly instantiated")
                .with_labels(labels),
        );
    }

    pub fn tyck_instantiate_wrong_arity(&mut self) {
        let labels = vec![Label::primary(self.at)];

        self.add(
            Diagnostic::error()
                .with_code(INSTANTIATE_WRONG_ARITY)
                .with_message("explicit instantiation has wrong number of arguments")
                .with_labels(labels),
        );
    }

    pub fn tyck_narrow_range(
        &mut self,
        (lo1, hi1): (Number, Number),
        (lo2, hi2): (Number, Number),
    ) {
        let labels = vec![Label::primary(self.at).with_message(format!(
            "expected a type narrower than '{lo1} .. {hi1}', but '{lo2} .. {hi2}' is wider"
        ))];

        self.add(
            Diagnostic::error()
                .with_code(NARROW_RANGE)
                .with_message("expected a narrower range type")
                .with_labels(labels),
        );
    }

    pub fn tyck_no_progress(&mut self) {
        let labels = vec![Label::primary(self.at)];

        self.add(
            Diagnostic::error()
                .with_code(NO_PROGRESS)
                .with_message("unable to make progress while typechecking")
                .with_labels(labels),
        );
    }

    pub fn tyck_not_a_fun(&mut self, ty: Option<impl Into<String>>) {
        let labels = if let Some(ty) = ty {
            vec![Label::primary(self.at)
                .with_message(format!("a function cannot initialize '{}'", ty.into()))]
        } else {
            vec![Label::primary(self.at)]
        };

        self.add(
            Diagnostic::error()
                .with_code(NOT_A_FUN)
                .with_message("cannot create non-function type with a function")
                .with_labels(labels),
        );
    }

    pub fn tyck_not_an_int(&mut self, ty: Option<impl Into<String>>) {
        let labels = if let Some(ty) = ty {
            vec![Label::primary(self.at)
                .with_message(format!("a number cannot initialize '{}'", ty.into()))]
        } else {
            vec![Label::primary(self.at)]
        };

        self.add(
            Diagnostic::error()
                .with_code(NOT_AN_INT)
                .with_message("cannot create non-numeric type with a number")
                .with_labels(labels),
        );
    }

    pub fn tyck_recursive_inference(&mut self, var: impl Into<String>, ty: impl Into<String>) {
        let labels = vec![Label::primary(self.at)];
        let notes = vec![format!(
            "the variable '{}' occurs inside the type '{}', so the two cannot be unified",
            var.into(),
            ty.into()
        )];

        self.add(
            Diagnostic::error()
                .with_code(RECURSIVE)
                .with_message("attempted to infer a recursive type")
                .with_labels(labels)
                .with_notes(notes),
        )
    }

    pub fn tyck_tuple_type(&mut self) {
        let labels = vec![Label::primary(self.at)];

        self.add(
            Diagnostic::error()
                .with_code(TUPLE_TYPE)
                .with_message("tuple destructuring is not currently supported for types")
                .with_labels(labels),
        );
    }
}
