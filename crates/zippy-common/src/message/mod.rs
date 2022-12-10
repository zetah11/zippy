mod source;

mod diagnostic;
mod elab;
mod lex;
mod parse;
mod resolve;
mod tyck;

pub use diagnostic::{Diagnostic, Label, LabelStyle, Severity};
pub use source::{File, Span};

#[derive(Debug, Default)]
pub struct Messages {
    pub msgs: Vec<Diagnostic>,
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

    pub fn is_empty(&self) -> bool {
        self.msgs.is_empty()
    }

    pub fn len(&self) -> usize {
        self.msgs.len()
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
    fn add(&mut self, diag: Diagnostic) {
        self.msgs.msgs.push(diag);
    }
}
