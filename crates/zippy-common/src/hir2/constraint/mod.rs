use super::because::Because;
use super::coerce::CoercionId;
use super::Type;
use crate::message::Span;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Constraint {
    IntType {
        at: Span,
        because: Because,
        ty: Type,
    },

    Assignable {
        at: Span,
        into: Type,
        from: Type,
        id: CoercionId,
    },
}
