use super::because::Because;
use super::Type;
use crate::message::Span;

#[derive(Debug)]
pub enum Constraint {
    IntType {
        at: Span,
        because: Because,
        ty: Type,
    },
}
