//! Sources are the points from which actual source code originate. Typically sources correspond
//! 1-to-1 with files, though the compiler library itself does not mandate such a distinction. In
//! principle, multiple sources could be part of a single file - the only requirement is that each
//! source must have some unique identifier. In the case where one source is one file, this
//! identifier could be its filepath and -name.
//!
//! This module also contains [`Span`]s, which are used to keep track of where and in which source
//! objects (like tokens or syntax trees) originate.

//use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::lang::Language;

/// A single source, with its identifier and the content it contains.
#[derive(Clone, Debug)]
pub struct Source {
    /// The unique name for this source.
    //pub name: String,

    /// The language of this source, that determines how the compiler treats it.
    pub lang: Language,
}

/// A lightweight and unique identifier to some source.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SourceId(usize);

impl SourceId {
    /// Create a `0` source id. Only availaible with tests.
    #[cfg(test)]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self(0)
    }

    /// Create a [`Span`] assosciated with this source.
    pub fn span(&self, start: usize, end: usize) -> Span {
        Span {
            source: *self,
            start,
            end,
        }
    }
}

/// A [`Span`] is some contiguous chunk of source text.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Span {
    /// The source that this span refers to. Because spans must be contiguous, one span can only
    /// ever refer to one source.
    pub source: SourceId,

    /// The (inclusive) character index for the start of the span.
    pub start: usize,

    /// The (exclusive) character index for the end of the span.
    pub end: usize,
}

impl Span {
    /// Combine `self` and `other` to get a span which contains both.
    pub fn combine(&self, other: &Self) -> Self {
        assert!(self.source == other.source);

        Self {
            source: self.source,
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    /// Returns `true` if `self` contains `other` (that is, the start of `self` is equal to or
    /// smaller than the start of `other` and the end of `self` is equal to or greater than the end
    /// of `other`).
    pub fn contains(&self, other: &Self) -> bool {
        self.start <= other.start && self.end >= other.end
    }

    /// Create a span with the `0` source id. Only available for tests.
    #[cfg(test)]
    pub fn new(start: usize, end: usize) -> Self {
        Self {
            source: SourceId(0),
            start,
            end,
        }
    }
}

/// Create a span with the `0` source id. Only available for tests.
#[cfg(test)]
pub fn span(start: usize, end: usize) -> Span {
    Span::new(start, end)
}

/// A thread safe generator of source ids.
#[derive(Debug, Default)]
pub struct SourceGen {
    curr: AtomicUsize,
}

impl SourceGen {
    /// Create a fresh new `SourceGen`.
    pub fn new() -> Self {
        Self {
            curr: AtomicUsize::new(0),
        }
    }

    /// Generate a completely fresh id.
    pub fn new_id(&self) -> SourceId {
        SourceId(self.curr.fetch_add(1, Ordering::Relaxed))
    }
}

/*
/// Stores different sources and their respective ids.
#[derive(Debug, Default)]
pub struct SourceStore {
    sources: HashMap<SourceId, Source>,
    curr_id: usize,
}

impl SourceStore {
    /// Create a new, empty source store.
    pub fn new() -> Self {
        Self {
            sources: HashMap::new(),
            curr_id: 0,
        }
    }

    /// Add a source to the store and get its id back.
    pub fn add(&mut self, source: Source) -> SourceId {
        let id = SourceId(self.curr_id);
        self.curr_id += 1;
        self.sources.insert(id, source);
        id
    }

    /// Get the source assosciated with this id. Returns `None` if the source assosciated with the
    /// given `id` has been removed.
    pub fn get(&self, id: &SourceId) -> Option<&Source> {
        self.sources.get(id)
    }

    /// Remove the source assosciated with the given id.
    pub fn remove(&mut self, id: &SourceId) {
        self.sources.remove(id);
    }
}

*/
