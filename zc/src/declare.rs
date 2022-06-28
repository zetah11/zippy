//! The declaration pass is responsible for two things:
//!
//! 1. *declaring* the names of items, variables, etc. such that they are
//!    visible to the name resolution pass
//! 2. collecting scope information - namely the shape of the scope tree and
//!    what names lie within it

mod decls;
mod scope;

use crate::name::Name;
use crate::parse::hir::HirData;
use crate::source::Span;

use scope::ScopeId;

/// The data assosciated with HIR trees after the declaration pass.
#[derive(Debug)]
pub struct DeclData;

impl HirData for DeclData {
    type Name = String;
    type Binding = (Name, Span);
    type Scope = ScopeId;
}
