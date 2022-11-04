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

    /// Look for a top-level name, such as an entry point.
    pub fn find_in(&self, ctx: Name, name: impl Into<String>) -> Option<Name> {
        self.lookup(&Path(Some(ctx), Actual::Lit(name.into())))
    }
}
