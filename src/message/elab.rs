use codespan_reporting::diagnostic::{Diagnostic, Label};

use super::MessageAdder;

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
}
