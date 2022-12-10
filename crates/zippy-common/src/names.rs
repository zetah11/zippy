use std::collections::HashMap;

use bimap::BiMap;

use crate::hir::BindId;
use crate::message::Span;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct GeneratedName(usize);

impl GeneratedName {
    pub fn to_string(&self, prefix: impl AsRef<str>) -> String {
        format!("{}{}", prefix.as_ref(), self.0)
    }
}

impl From<GeneratedName> for String {
    fn from(name: GeneratedName) -> Self {
        format!("_t{}", name.0)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Name(usize);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Path(pub Option<Name>, pub Actual);

impl Path {
    pub fn new(ctx: Name, actual: Actual) -> Self {
        Self(Some(ctx), actual)
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Actual {
    Root,
    Lit(String),
    Scope(BindId),
    Generated(GeneratedName),
}

#[derive(Debug, Default)]
pub struct Names {
    names: BiMap<Name, Path>,
    decls: HashMap<Name, Span>,
    curr_gen: usize,
}

impl Names {
    pub fn new() -> Self {
        Self {
            names: BiMap::new(),
            decls: HashMap::new(),
            curr_gen: 0,
        }
    }

    pub fn root(&mut self) -> Name {
        self.add(Span::new(0, 0, 0), Path(None, Actual::Root))
    }

    pub fn add(&mut self, at: Span, name: Path) -> Name {
        if let Some(id) = self.names.get_by_right(&name) {
            *id
        } else {
            let id = Name(self.names.len());
            self.names.insert(id, name);
            self.decls.insert(id, at);
            id
        }
    }

    /// Generate a unique name, optionally at a given path.
    pub fn fresh(&mut self, at: Span, ctx: Name) -> Name {
        let id = GeneratedName(self.curr_gen);
        self.curr_gen += 1;

        self.add(at, Path(Some(ctx), Actual::Generated(id)))
    }

    pub fn get_path(&self, name: &Name) -> &Path {
        // Only one `Names` should be able to produce names, so this should never fail.
        self.names.get_by_left(name).unwrap()
    }

    pub fn get_span(&self, name: &Name) -> Span {
        *self.decls.get(name).unwrap()
    }

    pub fn lookup(&self, name: &Path) -> Option<Name> {
        self.names.get_by_right(name).copied()
    }

    /// Create a fresh name based on `name`, but with `target` replaced by `new` in its path.
    /// If this has no target to replace, a fresh name will be generated and inserted after the root, meaning this
    /// function always returns a new name.
    pub fn rebase(&mut self, name: &Name, target: &Name, new: &Name) -> Name {
        if name == target {
            *new
        } else {
            let Path(ctx, actual) = self.get_path(name);

            if let Some(ctx) = ctx {
                let ctx = *ctx;
                let actual = actual.clone();
                let span = self.get_span(name);

                let new_ctx = self.rebase(&ctx, target, new);
                self.add(span, Path(Some(new_ctx), actual))
            } else {
                let span = self.get_span(new);
                self.fresh(span, *name)
            }
        }
    }

    /// Look for a top-level name, such as an entry point.
    pub fn find_in(&self, ctx: Name, name: impl Into<String>) -> Option<Name> {
        self.lookup(&Path(Some(ctx), Actual::Lit(name.into())))
    }
}
