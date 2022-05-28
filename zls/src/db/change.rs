use std::sync::Arc;

use zc::source::{Source, SourceId};

/// Some change to the inputs to the compiler.
#[derive(Debug)]
pub enum Change {
    /// The content of the source file `at` changed.
    NewContent {
        /// The id for the source for which the change occured.
        at: SourceId,

        /// The new contents of the source.
        data: Arc<String>,
    },

    /// Assosciate an id and source metadata with the given uri.
    SourceData {
        /// The id for the given source.
        id: SourceId,

        /// The source metadata for the given source.
        source: Source,
    },
}
