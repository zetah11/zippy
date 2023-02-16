use super::{Diagnostic, Label, MessageAdder};

const RECURSIVE_KINDS: &str = "EK00";
const INCOMPATIBLE_KINDS: &str = "EK01";
const NOT_A_KIND: &str = "EK02";

impl MessageAdder<'_> {
    pub fn kick_recursive_kind(&mut self) {
        let labels = vec![Label::primary(self.at).with_message(
            "the kind of this type ends up referencing itself, which is not allowed",
        )];

        self.add(
            Diagnostic::error()
                .with_code(RECURSIVE_KINDS)
                .with_message("recursive kinds are not allowed")
                .with_labels(labels),
        );
    }

    pub fn kick_incompatible_kinds(
        &mut self,
        expected: impl Into<String>,
        actual: impl Into<String>,
    ) {
        let labels = vec![Label::primary(self.at).with_message(format!(
            "expected \"{}\" but got \"{}\"",
            expected.into(),
            actual.into()
        ))];

        self.add(
            Diagnostic::error()
                .with_code(INCOMPATIBLE_KINDS)
                .with_message("incompatible kinds")
                .with_labels(labels),
        );
    }

    pub fn kick_not_a_kind(&mut self) {
        let labels = vec![Label::primary(self.at)];

        self.add(
            Diagnostic::error()
                .with_code(NOT_A_KIND)
                .with_message("expected a kind")
                .with_labels(labels),
        );
    }
}
