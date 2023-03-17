use std::collections::HashMap;

use zippy_common::invalid::Reason;
use zippy_common::names::{ItemName, RawName};
use zippy_common::source::Span;

use crate::ast::Clusivity;
use crate::flattened::{Module, TypeExpression};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Type {
    Trait {
        values: HashMap<ItemName, Template>,
    },

    Range {
        clusivity: Clusivity,
        lower: TypeExpression,
        upper: TypeExpression,

        /// The module containing the two bounds.
        module: Module,
    },

    Var(UnifyVar),
    Unit,
    Invalid(Reason),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Template {
    pub ty: Type,
}

impl Template {
    /// Create a new monomorphic type.
    pub fn mono(ty: Type) -> Self {
        Self { ty }
    }

    /// Create a new template with the same parameters but a different inner
    /// type.
    pub fn with_type(&self, ty: Type) -> Self {
        Self { ty }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Constraint {
    /// Constrain `from` to be equal to or coercible to `into`.
    Assignable {
        at: Span,
        id: CoercionVar,
        into: Type,
        from: Type,
    },

    /// Constrain the two given types to be equal.
    Equal(Span, Type, Type),

    /// Constrain `target` to be the equal to the type of the field `field` on
    /// the type `of`.
    Field {
        at: Span,
        target: Type,
        of: Type,
        field: RawName,
    },

    /// Constrain the type to be an instantiation of the given template. This
    /// requires solving the template first, to make sure no type parameters
    /// escape, and *then* instantiating them with unification vars.
    Instantiated(Span, Type, Template),

    /// Constrain the type to be unit-like.
    UnitLike(Span, Type),

    /// Constrain the type to be numeric.
    Numeric(Span, Type),

    /// Constrain the type to be some string type.
    Textual(Span, Type),

    /// Constrain the type to be numeric, or `number` if it is ambiguous.
    TypeNumeric(Span, Type),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UnifyVar {
    pub span: Span,
    pub count: usize,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CoercionVar {
    pub span: Span,
    pub count: usize,
}
