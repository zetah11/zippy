//! Defines a query which describes the inputs to the compiler.

pub use inputs_def::{Inputs, InputsStorage};

mod inputs_def {
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
    }
}
