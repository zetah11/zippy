use std::collections::HashMap;

use lsp_types::{Diagnostic, Url};

use zippy_common::messages::Messages;
use zippy_common::source::project::module_name_from_source;
use zippy_frontend::names::declare::declared_names;
use zippy_frontend::parser::get_ast;

use super::Backend;
use crate::pretty::Prettier;

impl Backend {
    /// Check the project and return a list of generated diagnostics.
    pub(super) fn check(&self) -> HashMap<Url, Vec<Diagnostic>> {
        let mut diagnostics: HashMap<Url, Vec<_>> = HashMap::new();
        let mut asts: HashMap<_, Vec<_>> = HashMap::new();

        let prettier = Prettier::new(&self.database)
            .with_full_name(false)
            .with_include_span(false);

        for source in self.database.sources.iter() {
            let ast = get_ast(&self.database, *source);
            let source_name = *ast.source(&self.database).name(&self.database);
            let module_name = module_name_from_source(&self.database, source_name);

            asts.entry(module_name).or_default().push(ast);

            for message in get_ast::accumulated::<Messages>(&self.database, *source) {
                let (url, diagnostic) = self.make_diagnostic(&prettier, message);
                diagnostics.entry(url).or_default().push(diagnostic);
            }
        }

        let modules: Vec<_> = asts
            .into_iter()
            .map(|(name, sources)| zippy_frontend::ast::Module::new(&self.database, name, sources))
            .collect();

        for module in modules {
            let _ = declared_names(&self.database, module);

            for message in declared_names::accumulated::<Messages>(&self.database, module) {
                let (url, diagnostic) = self.make_diagnostic(&prettier, message);
                diagnostics.entry(url).or_default().push(diagnostic);
            }
        }

        let _ = modules;

        diagnostics
    }
}
