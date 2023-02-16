use crate::message::Span;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Because {
    /// The type constraint is due to an explicit type annotation.
    Annotation(Span),
    /// This type constraint is due to this expression being used as the
    /// argument of a function.
    Argument(Span),
    /// This type constraint is due to this expression being called with the
    /// spanned argument.
    Called(Span),
}
