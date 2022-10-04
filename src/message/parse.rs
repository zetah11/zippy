use codespan_reporting::diagnostic::{Diagnostic, Label};

use super::MessageAdder;

const BASE_EXPR: &str = "EP00";
//const NOT_AN_EXPR: &str = "EP01";
const NOT_A_PAT: &str = "EP02";
const NOT_A_TYPE: &str = "EP03";
const RANGE_NOT_AN_INT: &str = "EP04";
const UNCLOSED_GROUP: &str = "EP05";

impl<'a> MessageAdder<'a> {
    pub fn parse_expected_base_expr(&mut self) {
        let labels = vec![Label::primary(self.at.file, self.at)
            .with_message("expected a name, number, or parenthesized expression")];

        self.add(
            Diagnostic::error()
                .with_code(BASE_EXPR)
                .with_message("expected a simple expression")
                .with_labels(labels),
        );
    }

    /*
    pub fn parse_not_an_expression(&mut self) {
        let labels = vec![Label::primary(self.at.file, self.at)];

        self.add(
            Diagnostic::error()
                .with_code(NOT_AN_EXPR)
                .with_message("expected an expression")
                .with_labels(labels),
        );
    }
    */

    pub fn parse_not_a_pattern(&mut self) {
        let labels = vec![Label::primary(self.at.file, self.at)];
        let notes = vec![String::from("a pattern is a name or a literal")];

        self.add(
            Diagnostic::error()
                .with_code(NOT_A_PAT)
                .with_message("expected a pattern")
                .with_labels(labels)
                .with_notes(notes),
        );
    }

    pub fn parse_not_a_type(&mut self) {
        let labels = vec![Label::primary(self.at.file, self.at)];

        self.add(
            Diagnostic::error()
                .with_code(NOT_A_TYPE)
                .with_message("expected a type")
                .with_labels(labels),
        );
    }

    pub fn parse_range_not_an_int(&mut self) {
        let labels = vec![Label::primary(self.at.file, self.at)];

        self.add(
            Diagnostic::error()
                .with_code(RANGE_NOT_AN_INT)
                .with_message("range types can only contain integer literals")
                .with_labels(labels),
        );
    }

    pub fn parse_unclosed_group(&mut self) {
        let labels = vec![Label::primary(self.at.file, self.at)];

        self.add(
            Diagnostic::error()
                .with_code(UNCLOSED_GROUP)
                .with_message("unclosed group")
                .with_labels(labels),
        );
    }
}
