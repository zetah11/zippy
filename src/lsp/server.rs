use crossbeam::channel::{Receiver, Sender};
use lsp_server::{Connection, Message, Notification, Request, Response};
use lsp_types::notification::{self, Notification as _};
use lsp_types::request::{self, Request as _};
use lsp_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    DidSaveTextDocumentParams, InitializeParams, ServerCapabilities, ServerInfo,
};

use super::client::Client;

/// The `Server` trait implements various LSP requests and notifications as
/// methods, which are dispatched and properly handled by [`LspServer`].
pub trait Server {
    /// `textDocument/didChange`
    fn did_change_text_document(&mut self, params: DidChangeTextDocumentParams);
    /// `textDocument/didClose`
    fn did_close_text_document(&mut self, params: DidCloseTextDocumentParams);
    /// `textDocument/didOpen`
    fn did_open_text_document(&mut self, params: DidOpenTextDocumentParams);
    /// `textDocument/didSave`
    fn did_save_text_document(&mut self, params: DidSaveTextDocumentParams);
    /// `shutdown`
    fn shutdown(&mut self);
}

pub enum LspError {
    Exit(i32),
    Err(anyhow::Error),
}

impl<E: std::error::Error + Sync + Send + 'static> From<E> for LspError {
    fn from(value: E) -> Self {
        Self::Err(anyhow::Error::new(value))
    }
}

/// This is responsible for managing the connection and communicating with the
/// client. It exposes various notifications and requests in the [`Server`]
/// trait which the backend implements. The server itself takes care of
/// sending responses to the client when necessary.
pub struct LspServer {
    connection: Connection,
}

impl LspServer {
    pub fn stdio() -> Self {
        let (connection, _) = Connection::stdio();
        Self { connection }
    }

    pub fn serve(self, mut server: impl InitServer) -> Result<(), LspError> {
        let (id, params) = self.connection.initialize_start()?;
        let params: InitializeParams = serde_json::from_value(params)?;

        let capabilities = server.initialize(params);
        let info = server.name().map(|name| ServerInfo {
            name,
            version: server.version(),
        });

        let params = serde_json::json!({
            "capabilities": capabilities,
            "serverInfo": info,
        });

        self.connection.initialize_finish(id, params)?;

        let client = Client::new(self.connection.sender.clone());
        MainLoop::new(
            self.connection.receiver,
            self.connection.sender,
            server.build(client),
        )
        .run()
    }
}

/// This trait is used for a server "builder", and is used during the initial
/// initialization of the server.
pub trait InitServer {
    type Server: Server;

    fn build(self, client: Client) -> Self::Server;

    /// Initialize the server, and return its capabilities.
    fn initialize(&mut self, _params: InitializeParams) -> ServerCapabilities {
        Default::default()
    }

    fn name(&self) -> Option<String> {
        None
    }

    fn version(&self) -> Option<String> {
        None
    }
}

/// This implements the main LSP loop, waiting for requests and notifications,
/// dispatching them to the server, and sending responses (when necessary).
struct MainLoop<S> {
    requests: Receiver<Message>,
    responses: Sender<Message>,
    server: S,

    requested_shutdown: bool,
}

impl<S: Server> MainLoop<S> {
    pub fn new(requests: Receiver<Message>, responses: Sender<Message>, server: S) -> Self {
        Self {
            requests,
            responses,
            server,

            requested_shutdown: true,
        }
    }

    pub fn run(mut self) -> Result<(), LspError> {
        while let Ok(message) = self.requests.recv() {
            match message {
                Message::Request(request) => self.handle_request(request)?,
                Message::Notification(notification) => self.handle_notification(notification)?,

                Message::Response(_) => unimplemented!(),
            }
        }

        Result::Ok(())
    }

    /// Dispatch on a notification sent to the server.
    fn handle_notification(&mut self, notification: Notification) -> Result<(), LspError> {
        match &notification.method[..] {
            m if m == notification::Exit::METHOD => {
                return Err(LspError::Exit(i32::from(!self.requested_shutdown)));
            }

            m if m == notification::DidChangeTextDocument::METHOD => {
                let params = notification.extract(notification::DidChangeTextDocument::METHOD)?;
                self.server.did_change_text_document(params);
            }

            m if m == notification::DidCloseTextDocument::METHOD => {
                let params = notification.extract(notification::DidCloseTextDocument::METHOD)?;
                self.server.did_open_text_document(params);
            }

            m if m == notification::DidOpenTextDocument::METHOD => {
                let params = notification.extract(notification::DidOpenTextDocument::METHOD)?;
                self.server.did_open_text_document(params);
            }

            m if m == notification::DidSaveTextDocument::METHOD => {
                let params = notification.extract(notification::DidSaveTextDocument::METHOD)?;
                self.server.did_save_text_document(params);
            }

            _ => {}
        }

        Result::Ok(())
    }

    /// Dispatch on and respond to a request sent to the server.
    fn handle_request(&mut self, request: Request) -> Result<(), LspError> {
        match &request.method[..] {
            m if m == request::Shutdown::METHOD => {
                self.requested_shutdown = true;
                self.server.shutdown();
                self.responses
                    .send(Message::Response(Response::new_ok(request.id, ())))
                    .expect("attempted to send response over closed channel");
            }

            _ => {}
        }

        Result::Ok(())
    }
}
