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

    pub fn elab_outside_range(&mut self, lo: i64, hi: i64) {
        let labels = vec![Label::primary(self.at.file, self.at)
            .with_message(format!("this value is outside '{lo} upto {hi}'"))];

        self.add(
            Diagnostic::error()
                .with_code(OUTSIDE_RANGE)
                .with_message("integer value outside allowed range")
                .with_labels(labels),
        );
    }
}
