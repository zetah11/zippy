use super::{Diagnostic, Label, MessageAdder, Messages, Span};

const REDECLARATION: &str = "ER00";
const UNKNOWN_NAME: &str = "ER01";
const NO_ENTRY_POINT: &str = "ER02";

impl Messages {
    pub fn resolve_no_entry_point(&mut self) {
        self.msgs.push(
            Diagnostic::error()
                .with_code(NO_ENTRY_POINT)
                .with_message("program has no entry point"),
        );
    }
}

impl<'a> MessageAdder<'a> {
    pub fn resolve_redeclaration(&mut self, prev: Span) {
        let labels = vec![
            Label::primary(self.at),
            Label::secondary(prev).with_message("previous declaration here"),
        ];

        self.add(
            Diagnostic::error()
                .with_code(REDECLARATION)
                .with_message("redeclaration of existing name")
                .with_labels(labels),
        );
    }

    pub fn resolve_unknown_name(&mut self, name: &str) {
        let labels = vec![Label::primary(self.at)];

        self.add(
            Diagnostic::error()
                .with_code(UNKNOWN_NAME)
                .with_message(format!("unresolved name '{}'", name))
                .with_labels(labels),
        );
    }
}
