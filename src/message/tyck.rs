use codespan_reporting::diagnostic::{Diagnostic, Label};

use super::MessageAdder;

const INCOMPATIBLE_TYPES: &str = "ET00";
const NARROW_RANGE: &str = "ET01";
const NOT_A_FUN: &str = "ET02";
const NOT_AN_INT: &str = "ET03";
const AMBIGUOUS: &str = "ET04";
const RECURSIVE: &str = "ET05";

impl<'a> MessageAdder<'a> {
    pub fn tyck_ambiguous(&mut self) {
        let labels = vec![Label::primary(self.at.file, self.at)
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
        let labels = match (expected, actual) {
            (Some(expected), Some(actual)) => vec![Label::primary(self.at.file, self.at)
                .with_message(format!(
                    "expected '{}', got '{}'",
                    expected.into(),
                    actual.into()
                ))],
            _ => Vec::new(),
        };

        self.add(
            Diagnostic::error()
                .with_code(INCOMPATIBLE_TYPES)
                .with_message("incompatible types")
                .with_labels(labels),
        );
    }

    pub fn tyck_narrow_range(&mut self, (lo1, hi1): (i64, i64), (lo2, hi2): (i64, i64)) {
        let labels = vec![Label::primary(self.at.file, self.at).with_message(format!(
            "expected a type narrower than '{lo1} .. {hi1}', but '{lo2} .. {hi2}' is wider"
        ))];

        self.add(
            Diagnostic::error()
                .with_code(NARROW_RANGE)
                .with_message("expected a narrower range type")
                .with_labels(labels),
        )
    }

    pub fn tyck_not_a_fun(&mut self, ty: Option<impl Into<String>>) {
        let labels = if let Some(ty) = ty {
            vec![Label::primary(self.at.file, self.at)
                .with_message(format!("a function cannot initialize '{}'", ty.into()))]
        } else {
            vec![Label::primary(self.at.file, self.at)]
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
            vec![Label::primary(self.at.file, self.at)
                .with_message(format!("a number cannot initialize '{}'", ty.into()))]
        } else {
            vec![Label::primary(self.at.file, self.at)]
        };

        self.add(
            Diagnostic::error()
                .with_code(NOT_AN_INT)
                .with_message("cannot create non-numeric type with a number")
                .with_labels(labels),
        );
    }

    pub fn tyck_recursive_inference(&mut self, var: impl Into<String>, ty: impl Into<String>) {
        let labels = vec![Label::primary(self.at.file, self.at)];
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
}
