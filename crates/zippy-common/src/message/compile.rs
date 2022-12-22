use super::{Diagnostic, Label, MessageAdder};

const UNCONSTRAINED_RANGE: &str = "EC00";

impl<'a> MessageAdder<'a> {
    pub fn compile_unconstrained_range(&mut self) {
        let labels = vec![Label::primary(self.at)];
        let notes = vec![
            "note: the compiler needs to know the range of possible values this expression can take, but because evaluation is disabled, it is unable to do so".into(),
            "help: try annotating this expression with a type".into(),
            "help: try moving this expression to a `let`-binding".into(),
        ];

        self.add(
            Diagnostic::error()
                .with_code(UNCONSTRAINED_RANGE)
                .with_message("unconstrained range bound")
                .with_labels(labels)
                .with_notes(notes),
        );
    }
}
