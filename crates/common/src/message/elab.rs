use super::{Diagnostic, Label, MessageAdder, Messages, Span};

const OUTSIDE_RANGE: &str = "EE00";
const CLOSURE: &str = "EE01";
const REQUIRES_INIT: &str = "EE02";

const REPORT_HOLE: &str = "HE00";

impl Messages {
    pub fn elab_closure_not_permitted(&mut self, free: impl Iterator<Item = Span>) {
        let labels = free.map(Label::secondary).collect();

        let notes = vec!["note: these variables are not defined inside the function".into()];

        self.msgs.push(
            Diagnostic::error()
                .with_code(CLOSURE)
                .with_message("closures are not permitted")
                .with_labels(labels)
                .with_notes(notes),
        );
    }
}

impl<'a> MessageAdder<'a> {
    pub fn elab_report_hole(&mut self, ty: impl Into<String>) {
        let labels = vec![Label::primary(self.at)];

        self.add(
            Diagnostic::help()
                .with_code(REPORT_HOLE)
                .with_message(format!("this hole has type '{}'", ty.into()))
                .with_labels(labels),
        );
    }

    pub fn elab_outside_range(&mut self, ty: impl Into<String>, off_by_one: bool) {
        let labels = vec![Label::primary(self.at).with_message(format!(
            "this value is outside the range of '{}'",
            ty.into()
        ))];

        let notes = if off_by_one {
            vec![String::from(
                "note: the upper bound is exclusive, so this value is not part of the type",
            )]
        } else {
            Vec::new()
        };

        self.add(
            Diagnostic::error()
                .with_code(OUTSIDE_RANGE)
                .with_message("integer value outside allowed range")
                .with_labels(labels)
                .with_notes(notes),
        );
    }

    pub fn elab_requires_init(&mut self) {
        let labels = vec![Label::primary(self.at)
            .with_message("this expression cannot be fully evaluated at compile time")];
        let notes = vec![
            "note: global values which require run-time initialization are not currently supported"
                .into(),
        ];

        self.add(
            Diagnostic::error()
                .with_code(REQUIRES_INIT)
                .with_message("global variable requires run-time initialization")
                .with_labels(labels)
                .with_notes(notes),
        );
    }
}
