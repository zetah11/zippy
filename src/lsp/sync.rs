use std::collections::HashSet;
use std::fs;

use lsp_types::{notification, Url};
use lsp_types::{MessageType, PublishDiagnosticsParams};

use super::Backend;

impl Backend {
    /// Check the project and publish the generated diagnostics.
    pub(super) fn check_and_publish(&mut self) {
        let diagnostics = self.check();

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

        let path = match uri.to_file_path() {
            Ok(path) => path,
            Err(()) => {
                self.client.log(
                    MessageType::ERROR,
                    format!("invalid file uri {}", uri.as_str()),
                );
                return;
            }
        };

        let content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(e) => {
                self.client.log(
                    MessageType::ERROR,
                    format!("error reading source {}: {e}", path.display()),
                );
                return;
            }
        };

        let name = self.path_to_source_name(path.clone());
        self.write_content(path, name, content);
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

        let path = match uri.to_file_path() {
            Ok(path) => path,
            Err(()) => {
                self.client.log(
                    MessageType::ERROR,
                    format!("invalid file uri {}", uri.as_str()),
                );
                return;
            }
        };

        let name = self.path_to_source_name(path.clone());
        self.write_content(path, name, content);
    }
}
