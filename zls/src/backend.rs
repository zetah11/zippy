//! The main driver which handles communication between the lsp client and the
//! compiler.

use std::sync::Arc;

use dashmap::DashMap;
use ropey::Rope;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use zc::inputs::Inputs;
use zc::source::{Source, SourceId};

use crate::db::{Change, Database};
use crate::file::{uri_to_language, FilenameMap};

/// The backend responds to LSP requests.
#[derive(Debug)]
pub struct Backend {
    client: Client,
    document_map: DashMap<SourceId, Rope>,
    database: Database,
    name_id_map: FilenameMap,
}

impl Backend {
    /// Create a new backend.
    pub fn new(client: Client) -> Self {
        Self {
            client,
            document_map: DashMap::new(),
            database: Database::new(),
            name_id_map: FilenameMap::new(),
        }
    }

    /// Get the id for the given url, or add it if it doesn't exist.
    fn get_or_add_id(&self, url: String) -> SourceId {
        let lang = uri_to_language(&url);

        let id = if let Some(id) = self.name_id_map.get_id(&url) {
            id
        } else {
            self.name_id_map.add(url)
        };

        self.database.update(Change::SourceData {
            id,
            source: Source { lang },
        });

        id
    }

    /// Update compiler state on text change.
    fn on_change(&self, id: SourceId, text: Option<&str>) {
        let rope = if let Some(rope) = self.document_map.get(&id) {
            rope
        } else {
            return;
        };

        let text = if let Some(text) = text {
            text
        } else if let Some(text) = rope.slice(..).as_str() {
            text
        } else {
            return;
        };

        self.database.update(Change::NewContent {
            at: id,
            data: Arc::new(String::from(text)),
        });
    }

    async fn analyze(&self, id: SourceId) {
        let snapshot = self.database.get_snapshot();
        let wc = snapshot.count_words(id);
        self.client
            .log_message(MessageType::INFO, format!("wc: {wc}"))
            .await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        let file_reg_options = FileOperationRegistrationOptions {
            filters: vec![FileOperationFilter {
                scheme: Some("file".into()),
                pattern: FileOperationPattern {
                    glob: "**/*.{z,zd,zs}".into(),
                    matches: Some(FileOperationPatternKind::File),
                    options: None,
                },
            }],
        };

        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "zls".into(),
                ..Default::default()
            }),
            capabilities: ServerCapabilities {
                rename_provider: Some(OneOf::Left(true)),
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                workspace: Some(WorkspaceServerCapabilities {
                    file_operations: Some(WorkspaceFileOperationsServerCapabilities {
                        did_rename: Some(file_reg_options.clone()),
                        did_delete: Some(file_reg_options),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file opened")
            .await;

        let id = self.get_or_add_id(params.text_document.uri.to_string());

        let rope = Rope::from_str(&params.text_document.text);
        self.document_map.insert(id, rope);

        self.on_change(id, Some(&params.text_document.text));
    }

    async fn did_delete_files(&self, params: DeleteFilesParams) {
        self.client
            .log_message(MessageType::INFO, "deleting files")
            .await;

        for file in params.files {
            self.client
                .log_message(MessageType::INFO, file.uri.clone())
                .await;

            if let Some(id) = self.name_id_map.get_id(&file.uri) {
                self.name_id_map.remove_id(&id);
            }
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file saved")
            .await;

        let id = self.get_or_add_id(params.text_document.uri.to_string());

        if let Some(text) = params.text {
            let rope = Rope::from_str(&text);
            self.document_map.insert(id, rope);

            self.on_change(id, Some(&text));
        } else {
            self.on_change(id, None);
        }
    }

    async fn did_rename_files(&self, params: RenameFilesParams) {
        self.client
            .log_message(MessageType::INFO, "renaming files")
            .await;

        for FileRename { old_uri, new_uri } in params.files {
            self.client
                .log_message(MessageType::INFO, format!("renamed {old_uri} to {new_uri}"))
                .await;

            assert!(self.name_id_map.get_id(&new_uri).is_none());
            if let Some(id) = self.name_id_map.get_id(&old_uri) {
                self.name_id_map.rename_id(&id, new_uri);
            }
        }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let id = self.get_or_add_id(params.text_document.uri.to_string());
        let rope = self.document_map.get_mut(&id);

        if let Some(mut rope) = rope {
            for change in params.content_changes {
                if let Some(range) = change.range {
                    rope.remove(range.start.character as usize..range.end.character as usize);
                    rope.insert(range.start.character as usize, &change.text);
                } else {
                    rope.remove(..);
                    rope.insert(0, &change.text);
                }
            }
        }

        self.on_change(id, None);
        self.analyze(id).await;
    }
}

#[cfg(never)]
fn offset_to_position(offset: usize, rope: &Rope) -> Option<Position> {
    let line = rope.try_char_to_line(offset).ok()?;
    let first_char = rope.try_line_to_char(line).ok()?;
    let column = offset - first_char;
    Some(Position::new(line as u32, column as u32))
}
