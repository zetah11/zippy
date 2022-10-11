use std::collections::HashMap;

use bimap::BiMap;

use crate::hir::BindId;
use crate::message::Span;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct GeneratedName(usize);

impl From<GeneratedName> for String {
    fn from(name: GeneratedName) -> Self {
        format!("_t{}", name.0)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Name(usize);

#[derive(Debug, Default)]
pub struct Names {
    names: BiMap<Name, Path>,
    decls: HashMap<Name, Span>,
    curr_gen: usize,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Path(pub Vec<Name>, pub Actual);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Actual {
    Lit(String),
    Scope(BindId),
    Generated(GeneratedName),
}

impl Names {
    pub fn new() -> Self {
        Self {
            names: BiMap::new(),
            decls: HashMap::new(),
            curr_gen: 0,
        }
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
    pub fn fresh(&mut self, at: Span, ctx: Option<Name>) -> Name {
        let id = GeneratedName(self.curr_gen);
        self.curr_gen += 1;

        self.add(at, Path(ctx.into_iter().collect(), Actual::Generated(id)))
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
    pub fn find_top_level(&self, name: impl Into<String>) -> Option<Name> {
        self.lookup(&Path(Vec::new(), Actual::Lit(name.into())))
    }
}
