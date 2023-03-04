use std::collections::HashMap;

use lsp_types::{Diagnostic, Url};

use zippy_common::messages::Messages;
use zippy_frontend::ast;
use zippy_frontend::names::declare::declared_names;
use zippy_frontend::parser::parse;

use super::Backend;
use crate::pretty::Prettier;

impl Backend {
    /// Check the project and return a list of generated diagnostics.
    pub(super) fn check(&mut self) -> HashMap<Url, Vec<Diagnostic>> {
        let mut diagnostics: HashMap<Url, Vec<_>> = HashMap::new();

        let mut messages = Vec::new();
        let sources: Vec<_> = self.database.sources.iter().map(|source| *source).collect();
        let asts = parse(&self.database, &mut messages, &sources);

        for (name, sources) in asts {
            let module = match self.database.ast_modules.get(&name) {
                Some(module) => *module,

                None => {
                    let module = ast::Module::new(&self.database, name, sources);
                    self.database.ast_modules.insert(name, module);
                    continue;
                }
            };

            module.set_sources(&mut self.database).to(sources);
        }

        for module in self.database.ast_modules.iter() {
            let _ = declared_names(&self.database, *module);
            messages.extend(declared_names::accumulated::<Messages>(
                &self.database,
                *module,
            ));
        }

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
