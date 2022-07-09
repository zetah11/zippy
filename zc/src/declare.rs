//! The declaration pass is responsible for two things:
//!
//! 1. *declaring* the names of items, variables, etc. such that they are
//!    visible to the name resolution pass
//! 2. collecting scope information - namely the shape of the scope tree and
//!    what names lie within it

mod declarer;

#[cfg(test)]
mod tests;

pub use decl::{DeclStorage, Declare};

use crate::name::{Bare, Name};
use crate::parse::hir::HirData;
use crate::scope::ScopeId;
use crate::source::Span;

/// The data assosciated with HIR trees after the declaration pass.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeclData;

impl HirData for DeclData {
    type Name = Bare;
    type Binding = (Name, Span);
    type Scope = ScopeId;
}

mod decl {
    use std::sync::Arc;

    use super::declarer::Declarer;
    use super::DeclData;
    use crate::name::{ActualName, Bare, Name, NameData, NameInterner};
    use crate::parse::hir::Decls;
    use crate::parse::ParsedData;
    use crate::parse::Parser;
    use crate::scope::{self, Scope};
    use crate::source::SourceId;

    type Scopes = scope::Scopes<(Bare, Name)>;

    /// See the [module-level documentation](crate::declare) for more.
    #[salsa::query_group(DeclStorage)]
    pub trait Declare: Parser + NameInterner {
        /// Declare the names in the given source.
        fn decl(&self, id: SourceId) -> (Arc<Decls<DeclData>>, Arc<Scopes>);
    }

    fn decl(db: &dyn Declare, id: SourceId) -> (Arc<Decls<DeclData>>, Arc<Scopes>) {
        let decls = db.parse_tree(id);
        let (decls, scopes) = declare_decls(decls, id, db);
        (Arc::new(decls), Arc::new(scopes))
    }

    fn declare_decls(
        decls: Arc<Decls<ParsedData>>,
        src: SourceId,
        names: &dyn Declare,
    ) -> (Decls<DeclData>, Scopes) {
        let mut scopes = Scopes::new();

        let top_id = scopes.make_id();
        scopes.add(
            top_id,
            Scope {
                parent: None,
                names: vec![],
            },
        );

        let mut declarer = Declarer::new(&mut scopes, names);

        let top = NameData::Root(ActualName::File(src));
        let top = names.intern_name(top);

        let decls = declarer.declare_decls((*decls).clone(), (top_id, top));
        (decls, scopes)
    }
}
