use codespan_reporting::diagnostic::{Diagnostic, Label};

use super::MessageAdder;

const OUTSIDE_RANGE: &str = "EE00";

const REPORT_HOLE: &str = "HE00";

impl<'a> MessageAdder<'a> {
    pub fn elab_report_hole(&mut self, ty: impl Into<String>) {
        let labels = vec![Label::primary(self.at.file, self.at)];

        self.add(
            Diagnostic::help()
                .with_code(REPORT_HOLE)
                .with_message(format!("this hole has type '{}'", ty.into()))
                .with_labels(labels),
        );
    }

    pub fn elab_outside_range(&mut self, ty: impl Into<String>, off_by_one: bool) {
        let labels = vec![Label::primary(self.at.file, self.at).with_message(format!(
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
}
