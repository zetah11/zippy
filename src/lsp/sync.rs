use std::collections::{HashMap, HashSet};

use lsp_types::{notification, Url};
use lsp_types::{MessageType, PublishDiagnosticsParams};

use zippy_common::messages::Messages;
use zippy_frontend::parser::get_ast;

use super::Backend;
use crate::project::SourceName;

impl Backend {
    /// Check the project and publish all generated diagnostics.
    pub(super) fn check(&mut self) {
        let mut diagnostics: HashMap<Url, Vec<_>> = HashMap::new();

        for source in self.database.sources.iter() {
            let _ = get_ast(&self.database, *source);

            for message in get_ast::accumulated::<Messages>(&self.database, *source) {
                let (url, diagnostic) = self.make_diagnostic(message);
                diagnostics.entry(url).or_default().push(diagnostic);
            }
        }

        let current_diagnostics: HashSet<_> = diagnostics.keys().cloned().collect();

        for (uri, diagnostics) in diagnostics {
            self.client
                .notify::<notification::PublishDiagnostics>(PublishDiagnosticsParams {
                    uri,
                    diagnostics,
                    version: None,
                });
        }

        // Documents which had diagnostics, but don't anymore need to be sent an
        // empty diagnostics array.
        for uri in self.has_diagnostics.drain() {
            if !current_diagnostics.contains(&uri) {
                self.client
                    .notify::<notification::PublishDiagnostics>(PublishDiagnosticsParams {
                        uri,
                        diagnostics: Vec::new(),
                        version: None,
                    });
            }
        }

        self.has_diagnostics = current_diagnostics;
    }

    /// Update the contents of the document such that it is in sync with the
    /// source itself.
    pub(super) fn update_document_from_source(&mut self, uri: Url) {
        if uri.scheme() != "file" {
            self.client.log(
                MessageType::ERROR,
                format!("cannot read non-file {}", uri.as_str()),
            );
            return;
        }

        let name = match uri.to_file_path() {
            Ok(path) => SourceName::new(path),
            Err(()) => {
                self.client.log(
                    MessageType::ERROR,
                    format!("invalid file uri {}", uri.as_str()),
                );
                return;
            }
        };

        let content = match self.database.read_source(name.clone()) {
            Ok(content) => content,
            Err(e) => {
                self.client.log(
                    MessageType::ERROR,
                    format!("error reading source {}: {e}", name.as_path().display()),
                );
                return;
            }
        };

        self.write_content(name, content);
    }

    /// Update the contents of the document from a string.
    pub(super) fn update_document(&mut self, uri: Url, content: String) {
        if uri.scheme() != "file" {
            self.client.log(
                MessageType::ERROR,
                format!("cannot read non-file {}", uri.as_str()),
            );
            return;
        }

        let name = match uri.to_file_path() {
            Ok(path) => SourceName::new(path),
            Err(()) => {
                self.client.log(
                    MessageType::ERROR,
                    format!("invalid file uri {}", uri.as_str()),
                );
                return;
            }
        };

        self.write_content(name, content);
    }

    /// Write the given content to the given file.
    fn write_content(&mut self, name: SourceName, content: String) {
        let source = match self.database.sources.get(&name) {
            Some(source) => *source,

            None => {
                self.database.add_source(name, content);
                return;
            }
        };

        source.set_content(&mut self.database).to(content);
    }
}
