//! Various bits and pieces related to pretty-printing of various things.

use crate::Database;

mod names;

pub struct Prettier<'db> {
    db: &'db Database,
    include_span: bool,
    full_name: bool,
}

impl<'db> Prettier<'db> {
    pub fn new(db: &'db Database) -> Self {
        Self {
            db,
            include_span: false,
            full_name: false,
        }
    }

    /// Whether to print the full path of every name.
    pub fn with_full_name(self, full_name: bool) -> Self {
        Self { full_name, ..self }
    }

    /// Whether to include span information in certain cases, like unnamable
    /// names.
    pub fn with_include_span(self, include_span: bool) -> Self {
        Self {
            include_span,
            ..self
        }
    }
}
