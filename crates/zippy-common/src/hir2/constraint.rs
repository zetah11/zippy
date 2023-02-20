use std::collections::HashMap;

use super::because::Because;
use super::coerce::CoercionId;
use super::{Type, UniVar};
use crate::message::Span;
use crate::names2::Name;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Constraint {
    NumberType {
        at: Span,
        because: Because,
        ty: Type,
    },

    Assignable {
        at: Span,
        id: CoercionId,
        into: Type,
        from: Type,
        subst: HashMap<UniVar, (HashMap<Name, Type>, Type)>,
    },

    Equal {
        at: Span,
        t: Type,
        u: Type,
        subst: HashMap<UniVar, (HashMap<Name, Type>, Type)>,
    },
}
