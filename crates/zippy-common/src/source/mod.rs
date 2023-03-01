use std::path::PathBuf;

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

/// A source is some "source" of code - almost always a file. It is identified
/// by some name, which, because it is almost always a file, is a [`PathBuf`],
/// and its contents.
#[salsa::input]
pub struct Source {
    #[id]
    #[return_ref]
    pub name: PathBuf,

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
