use super::{Diagnostic, Label, MessageAdder};

const BASE_EXPR: &str = "EP00";
const DECLARATION: &str = "EP01";
const DISALLOWED_IMPLICITS: &str = "EP08";
const EXPR: &str = "EP10";
const GENERIC_LAMBDA: &str = "EP09";
const NOT_A_PAT: &str = "EP02";
const NOT_A_TYPE: &str = "EP03";
const NOT_A_TYPE_NAME: &str = "EP07";
const TYPE_IMPLICITS: &str = "EP11";
const UNCLOSED_GROUP: &str = "EP05";
const UNCLOSED_IMPLICITS: &str = "EP06";

impl<'a> MessageAdder<'a> {
    pub fn parse_expected_base_expr(&mut self) {
        let labels = vec![Label::primary(self.at)
            .with_message("expected a name, number, or parenthesized expression")];

        self.add(
            Diagnostic::error()
                .with_code(BASE_EXPR)
                .with_message("expected a simple expression")
                .with_labels(labels),
        );
    }

    pub fn parse_expected_expr(&mut self) {
        let labels = vec![Label::primary(self.at)];

        self.add(
            Diagnostic::error()
                .with_code(EXPR)
                .with_message("expected an expression")
                .with_labels(labels),
        );
    }

    pub fn parse_disallowed_implicits(&mut self) {
        let labels = vec![Label::primary(self.at)];
        let notes = vec!["note: implicit list only allowed right after function name".into()];

        self.add(
            Diagnostic::error()
                .with_code(DISALLOWED_IMPLICITS)
                .with_message("implicit list not allowed in this position")
                .with_labels(labels)
                .with_notes(notes),
        );
    }

    pub fn parse_expected_declaration(&mut self) {
        let labels =
            vec![Label::primary(self.at).with_message("expected a value or type definition")];

        self.add(
            Diagnostic::error()
                .with_code(DECLARATION)
                .with_message("expected a declaration")
                .with_labels(labels),
        );
    }

    pub fn parse_generic_lambda(&mut self) {
        let labels = vec![Label::primary(self.at)];

        self.add(
            Diagnostic::error()
                .with_code(GENERIC_LAMBDA)
                .with_message("lambdas may not have implicit parameters")
                .with_labels(labels),
        );
    }

    pub fn parse_not_a_pattern(&mut self) {
        let labels = vec![Label::primary(self.at)];
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
        let labels = vec![Label::primary(self.at)];

        self.add(
            Diagnostic::error()
                .with_code(NOT_A_TYPE)
                .with_message("expected a type")
                .with_labels(labels),
        );
    }

    pub fn parse_not_a_type_name(&mut self) {
        let labels = vec![Label::primary(self.at)];

        self.add(
            Diagnostic::error()
                .with_code(NOT_A_TYPE_NAME)
                .with_message("expected a type name")
                .with_labels(labels),
        );
    }

    pub fn parse_types_take_no_implicits(&mut self) {
        let labels = vec![Label::primary(self.at)];

        self.add(
            Diagnostic::error()
                .with_code(TYPE_IMPLICITS)
                .with_message("types cannot have implicit arguments")
                .with_labels(labels),
        );
    }

    pub fn parse_unclosed_group(&mut self) {
        let labels = vec![Label::primary(self.at)];

        self.add(
            Diagnostic::error()
                .with_code(UNCLOSED_GROUP)
                .with_message("unclosed group")
                .with_labels(labels),
        );
    }

    pub fn parse_unclosed_implicits(&mut self) {
        let labels = vec![Label::primary(self.at)];
        let notes = vec!["note: implicits are listed inbetween two pipes ('fun f |T, U|')".into()];

        self.add(
            Diagnostic::error()
                .with_code(UNCLOSED_IMPLICITS)
                .with_message("unclosed implicits list")
                .with_labels(labels)
                .with_notes(notes),
        );
    }
}
