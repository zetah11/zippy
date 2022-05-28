//! Defines a query which describes the inputs to the compiler.

#![allow(missing_docs)]

use std::sync::Arc;

use crate::source::{Source, SourceId};

#[salsa::query_group(InputsStorage)]
pub trait Inputs {
    /// Get the contents of the file with the given uri.
    #[salsa::input]
    fn input_file(&self, id: SourceId) -> Arc<String>;

    /// Get the source metadata for the source id.
    #[salsa::input]
    fn source(&self, id: SourceId) -> Source;

    /// Count the number of words for the document with the given uri.
    fn count_words(&self, id: SourceId) -> usize;
}

fn count_words(db: &dyn Inputs, uri: SourceId) -> usize {
    let text = db.input_file(uri);
    text.split_ascii_whitespace().count()
}
