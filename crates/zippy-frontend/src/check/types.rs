use std::collections::HashMap;

use zippy_common::invalid::Reason;
use zippy_common::names::ItemName;
use zippy_common::source::Span;

use crate::ast::Clusivity;
use crate::flattened::{Module, TypeExpression};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Constraint {
    /// Constrain `from` to be equal to or coercible to `into`.
    Assignable { at: Span, into: Type, from: Type },

    /// Constrain the two given types to be equal.
    Equal(Span, Type, Type),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UnifyVar {
    pub span: Span,
    pub count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Type {
    Trait {
        values: HashMap<ItemName, Type>,
    },

    Range {
        clusivity: Clusivity,
        lower: TypeExpression,
        upper: TypeExpression,

        /// The module containing the two bounds.
        module: Module,
    },

    Var(UnifyVar),
    Invalid(Reason),
}
