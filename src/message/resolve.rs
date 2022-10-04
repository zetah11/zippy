use codespan_reporting::diagnostic::{Diagnostic, Label};

use super::{MessageAdder, Span};

const REDECLARATION: &str = "ER00";
const UNKNOWN_NAME: &str = "ER01";

impl<'a> MessageAdder<'a> {
    pub fn resolve_redeclaration(&mut self, prev: Span) {
        let labels = vec![
            Label::primary(self.at.file, self.at),
            Label::secondary(prev.file, prev).with_message("previous declaration here"),
        ];

        self.add(
            Diagnostic::error()
                .with_code(REDECLARATION)
                .with_message("redeclaration of existing name")
                .with_labels(labels),
        );
    }

    pub fn resolve_unknown_name(&mut self) {
        let labels = vec![Label::primary(self.at.file, self.at)];

        self.add(
            Diagnostic::error()
                .with_code(UNKNOWN_NAME)
                .with_message("unresolved name")
                .with_labels(labels),
        );
    }
}
