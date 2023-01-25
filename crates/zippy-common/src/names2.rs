//! This module contains representations of fully qualified names, which are
//! globally unambiguous. A qualified name is effectively a literal name (like
//! a string appearing in the source or a generated one) together with a path
//! from the root through all the names that "contain" this one.

use crate::hir::BindId;
use crate::message::Span;

/// A literal name or a generated one.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum NamePart {
    Source(String),

    /// A name identified by its scope.
    Scope(BindId),

    /// A name identified by its span.
    Spanned(Span),
}

/// A globally unambiguous name.
#[salsa::interned]
pub struct Name {
    pub path: Option<Name>,
    #[return_ref]
    pub name: NamePart,
}
