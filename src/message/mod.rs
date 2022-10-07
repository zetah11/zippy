mod source;

mod elab;
mod lex;
mod parse;
mod resolve;
mod tyck;

pub use source::{File, Span};

use codespan_reporting::diagnostic::Diagnostic;

#[derive(Debug, Default)]
pub struct Messages {
    pub msgs: Vec<Diagnostic<usize>>,
}

impl Messages {
    pub fn new() -> Self {
        Self { msgs: Vec::new() }
    }

    #[must_use]
    pub fn at(&mut self, span: Span) -> MessageAdder {
        MessageAdder {
            msgs: self,
            at: span,
        }
    }

    pub fn merge(&mut self, other: Messages) {
        self.msgs.extend(other.msgs);
    }
}

#[derive(Debug)]
pub struct MessageAdder<'a> {
    msgs: &'a mut Messages,
    at: Span,
}

impl<'a> MessageAdder<'a> {
    fn add(&mut self, diag: Diagnostic<usize>) {
        self.msgs.msgs.push(diag);
    }
}
