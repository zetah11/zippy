use crate::message::Span;

#[derive(Clone, Debug)]
pub enum Because {
    Annotation(Span),
    Inferred(Span, Option<Box<Because>>),
    Unified(Span),
}
