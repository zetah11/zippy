//! Name resolution is the process of (partially) finding out what item a name
//! may refer to. For certain constructs, this won't give an answer because a
//! name may either refer to different things, or its definition depends on the
//! type of a variable.
//!
//! In the simplest cases, this turns code like
//!
//! ```z
//! let x: Int = 5
//! fun f(y) => x + y
//! type Int = 0 upto 100
//! ```
//!
//! into something like this:
//!
//! ```z
//! -- names:
//! -- $0 -> x
//! -- $1 -> f
//! -- $2 -> $1.y
//! -- $3 -> Int
//!
//! let $0: $3 = 5
//! fun $1($2) => $0 + $2
//! type $3 = 0 upto 100
//! ```
//!
//! which makes certain things, like type checking, easier to perform, since
//! names are now globally unique. In computer science jargon, we may call this
//! process alpha renaming.
//!
//! As mentioned though, certain constructs make name resolution harder. In
//! particular, dot syntax is tricky to work with:
//!
//! ```z
//! x.f(y)
//! ```
//!
//! If `x` is a class with a field `f`, then this expression calls that field.
//! However, if `x` doesn't have that field, or isn't a class, then this
//! expression calls the function `f` which is found through normal scoping
//! rules. All of this means that doing name resolution here requires type
//! information, which we don't yet have! Because of this, the name resolver
//! may emit something like this the above example:
//!
//! ```z
//! $1.<$2 or field "f">($3)
//! ```
//!
//! This gives the typing pass the responsibility of disambiguating the `.f`.
//!
//! # Declarations
//!
//! Names and items may be (highly) mutually recursive, so the resolution pass
//! does actually consist of two passes, this being the second and
//! [declare](crate::declare) being the first. The first pass is responsible for
//! just collecting all the names, while this pass is responsible for resolving
//! names based on that information following scoping rules.

mod resolver;

pub use resl::{Resolve, ResolveStorage};

use crate::name::{Bare, Name};
use crate::parse::hir::HirData;
use crate::source::Span;

/// The data assosciated with a HIR tree after the resolution pass.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedData;

impl HirData for ResolvedData {
    type Name = ResolvedName;
    type Binding = (Name, Bare, Span);
    type Scope = ();
}

/// A resolved name.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ResolvedName {
    /// A name that is potentially a field. If it is not, then it refers to the
    /// name `ifnot`.
    MaybeField {
        /// The actual name as it appears in the source, which is used to look
        /// up
        name: Bare,

        /// The lexically scoped name to use if this does not refer to a field.
        ifnot: Name,
    },

    /// An fully, unambiguously resolved name.
    Resolved(Name),
}

mod resl {
    use std::sync::Arc;

    use super::resolver::Resolver;
    use super::ResolvedData;
    use crate::declare::{DeclData, Declare};
    use crate::message::Message;
    use crate::name::{Bare, Name, NameInterner};
    use crate::parse::hir::Decls;
    use crate::scope::Scopes;
    use crate::source::SourceId;

    /// See the [module-level documentation](crate::resolve) for more.
    #[salsa::query_group(ResolveStorage)]
    pub trait Resolve: Declare + NameInterner {
        /// Perform name resolution on the given source.
        fn resolve(&self, id: SourceId) -> (Arc<Decls<ResolvedData>>, Arc<Vec<Message>>);

        /// Get the parse tree for the given source following name resolution.
        fn resolve_tree(&self, id: SourceId) -> Arc<Decls<ResolvedData>>;

        /// Get the name resolution errors for the given source.
        fn resolve_errs(&self, id: SourceId) -> Arc<Vec<Message>>;
    }

    fn resolve(db: &dyn Resolve, id: SourceId) -> (Arc<Decls<ResolvedData>>, Arc<Vec<Message>>) {
        let (decls, lexical) = db.decl(id);
        let (decls, errs) = resolve_decls(decls, lexical, db);
        (Arc::new(decls), Arc::new(errs))
    }

    fn resolve_tree(db: &dyn Resolve, id: SourceId) -> Arc<Decls<ResolvedData>> {
        db.resolve(id).0
    }

    fn resolve_errs(db: &dyn Resolve, id: SourceId) -> Arc<Vec<Message>> {
        db.resolve(id).1
    }

    fn resolve_decls(
        decls: Arc<Decls<DeclData>>,
        lexical: Arc<Scopes<(Bare, Name)>>,
        names: &dyn Resolve,
    ) -> (Decls<ResolvedData>, Vec<Message>) {
        let mut resolver = Resolver::new(&lexical, names);
        let decls = resolver.resolve_decls((*decls).clone(), false);
        let errs = resolver.get_errs();
        (decls, errs)
    }
}
