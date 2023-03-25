use std::collections::HashMap;

use lsp_types::{Diagnostic, Url};

use zippy_frontend::check;

use super::Backend;
use crate::pretty::Prettier;

impl Backend {
    /// Check the project and return a list of generated diagnostics.
    pub(super) fn check(&self) -> HashMap<Url, Vec<Diagnostic>> {
        let mut diagnostics: HashMap<Url, Vec<_>> = HashMap::new();

        let mut messages = Vec::new();
        let _ = check::check(&self.database, &mut messages, self.database.get_modules());

        let prettier = Prettier::new(&self.database)
            .with_full_name(false)
            .with_include_span(false);

        for message in messages {
            let (url, diagnostic) = self.make_diagnostic(&prettier, message);
            diagnostics.entry(url).or_default().push(diagnostic);
        }

        diagnostics
    }
}
