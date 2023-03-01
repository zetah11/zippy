mod client;
mod diagnostic;
mod format;
mod server;
mod sync;

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use lsp_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    InitializeParams, MessageType, SaveOptions, ServerCapabilities, TextDocumentSyncCapability,
    TextDocumentSyncKind, TextDocumentSyncOptions, TextDocumentSyncSaveOptions, Url,
};

use self::client::Client;
use self::server::{InitServer, LspError, LspServer, Server};
use crate::project::get_project_sources;
use crate::{meta, Database};

/// Run the compiler as a language server on stdio. This function may exit the
/// process if given the `exit` notification.
pub fn lsp() -> anyhow::Result<()> {
    let backend = BackendBuilder::new();

    match LspServer::stdio().serve(backend) {
        Ok(()) => Ok(()),
        Err(LspError::Err(e)) => Err(e),
        Err(LspError::Exit(code)) => std::process::exit(code),
    }
}

/// The backend builder is responsible for initializing the language server
/// backend.
struct BackendBuilder {
    root: Option<PathBuf>,
}

impl BackendBuilder {
    pub fn new() -> Self {
        Self { root: None }
    }
}

impl InitServer for BackendBuilder {
    type Server = Backend;

    fn build(self, client: Client) -> Self::Server {
        let root = self
            .root
            .unwrap_or_else(|| std::env::current_dir().expect("no sensible project root"));

        Backend::new(root, client)
    }

    fn initialize(&mut self, params: InitializeParams) -> ServerCapabilities {
        self.root = params
            .workspace_folders
            .and_then(|mut folders| folders.pop().map(|wf| wf.uri))
            .or(params.root_uri)
            .and_then(|uri| uri.to_file_path().ok());

        ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Options(
                TextDocumentSyncOptions {
                    open_close: Some(true),
                    change: Some(TextDocumentSyncKind::FULL),
                    save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
                        include_text: Some(true),
                    })),
                    ..Default::default()
                },
            )),

            ..Default::default()
        }
    }

    fn name(&self) -> Option<String> {
        Some(meta::COMPILER_NAME.into())
    }

    fn version(&self) -> Option<String> {
        Some(format!("{}-{}", meta::VERSION, meta::TAG))
    }
}

/// The language server backend is the main body of the language server and is
/// responsible for keeping sources in sync, checking and publishing
/// diagnostics, communicating with the language client, and so on.
///
/// It contains a [`Database`] which is used to keep track of sources and
/// perform all the checks, and a [`Client`] which is used to send logs and
/// notifications to the language client.
struct Backend {
    client: Client,
    database: Database,

    /// Diagnostics in the language server protocol are (primarily) published by
    /// the server. A file without any diagnostics can be "cleared" by sending
    /// a `textDocument/publishDiagnostics` notification to the client with an
    /// empty diagnostics list. This keeps track of the documents which
    /// currently do have diagnostics such that we can clear them later.
    has_diagnostics: HashSet<Url>,
}

impl Backend {
    /// Create a new language server backend in the given project root for the
    /// given client.
    pub fn new(root: impl AsRef<Path>, mut client: Client) -> Self {
        let database = Database::new();

        for source in get_project_sources(root) {
            let content = match database.read_source(source.clone()) {
                Ok(content) => content,
                Err(e) => {
                    client.log(
                        MessageType::ERROR,
                        format!("Error initializing with file: {e}"),
                    );
                    continue;
                }
            };

            database.add_source(source, content);
        }

        Self {
            client,
            database,
            has_diagnostics: HashSet::new(),
        }
    }
}

impl Server for Backend {
    fn did_change_text_document(&mut self, mut params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;

        if params.content_changes.len() != 1 {
            self.client.log(
                MessageType::ERROR,
                format!("only full updates are supported (on {})", uri.as_str()),
            );
            return;
        };

        let content = params.content_changes.pop().unwrap();
        self.update_document(uri, content.text);
        self.check();
    }

    fn did_close_text_document(&mut self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        self.update_document_from_source(uri);
        self.check();
    }

    fn did_open_text_document(&mut self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;

        self.client
            .message(MessageType::INFO, format!("opened {}", uri.as_str()));

        let content = params.text_document.text;
        self.update_document(uri, content);
        self.check();
    }

    fn did_save_text_document(&mut self, params: lsp_types::DidSaveTextDocumentParams) {
        let uri = params.text_document.uri;

        if let Some(content) = params.text {
            self.update_document(uri, content);
        } else {
            self.update_document_from_source(uri);
        }

        self.check();
    }

    fn shutdown(&mut self) {}
}
