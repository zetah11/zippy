use crate::message::Span;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Because {
    Annotation(Span),
    Inferred(Span, Option<Box<Because>>),
    Unified(Span),
}
