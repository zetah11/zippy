use super::{Diagnostic, Label, MessageAdder};

const UNSUPPORTED_CONVENTION: &str = "ET00";

impl MessageAdder<'_> {
    pub fn compile_unsupported_convention(
        &mut self,
        target: impl AsRef<str>,
        convention: impl AsRef<str>,
    ) {
        let labels = vec![Label::primary(self.at)];

        self.add(
            Diagnostic::error()
                .with_code(UNSUPPORTED_CONVENTION)
                .with_message(format!(
                    "target '{}' does not support calling convention '{}'",
                    target.as_ref(),
                    convention.as_ref(),
                ))
                .with_labels(labels),
        );
    }
}
