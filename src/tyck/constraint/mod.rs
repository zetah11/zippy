use super::Type;
use crate::message::Span;

#[derive(Debug)]
pub enum Constraint {
    IntType(Span, Type),
}
