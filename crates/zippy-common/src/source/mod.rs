use crate::names::ItemName;

pub mod project;

/// A module is a collection of sources whose declared items share a namespace.
#[salsa::input]
pub struct Module {
    #[id]
    pub name: ItemName,

    #[return_ref]
    pub sources: Vec<Source>,
}

/// A project represents a bunch of sources grouped by some common "root name" -
/// the name of this project.
#[salsa::input]
pub struct Project {
    #[return_ref]
    pub name: String,
}

/// A span represents a continuous range of text in a particular source file.
/// Spans can be combined using the `+` operator to create the smallest
/// continuous span containing both.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub source: Source,
}

impl std::ops::Add for Span {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        assert!(self.source == rhs.source);
        Self {
            source: self.source,
            start: self.start.min(rhs.start),
            end: self.end.max(rhs.end),
        }
    }
}

impl std::ops::AddAssign for Span {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

/// A source is some "source" of code - almost always a file - identified by a
/// [`SourceName`].
#[salsa::input]
pub struct Source {
    #[id]
    #[return_ref]
    pub name: SourceName,

    #[return_ref]
    pub content: String,
}

impl Source {
    pub fn span(&self, start: usize, end: usize) -> Span {
        Span {
            source: *self,
            start,
            end,
        }
    }
}

/// The name of a source. Source names are identified by the project they belong
/// to and a list of "parts" to its name. The module a source belongs to is
/// derived from its path, so it should ideally be something usable from code,
/// but in principle it may be anything. For a file, the parts could for
/// instance be the names of every parent directory up to some project root.
#[salsa::input]
pub struct SourceName {
    pub project: Project,

    #[return_ref]
    pub parts: Vec<String>,
}
