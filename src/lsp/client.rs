use crossbeam::channel::Sender;
use lsp_server::{Message, Notification};
use lsp_types::notification::{self, Notification as _};
use lsp_types::{LogMessageParams, MessageType, ShowMessageParams};

#[derive(Clone, Debug)]
pub struct Client {
    sender: Sender<Message>,
}

impl Client {
    pub fn new(sender: Sender<Message>) -> Self {
        Self { sender }
    }

    /// Send a log message to the client.
    pub fn log(&mut self, ty: MessageType, message: impl Into<String>) {
        self.sender
            .send(Message::Notification(Notification::new(
                notification::LogMessage::METHOD.to_string(),
                LogMessageParams {
                    typ: ty,
                    message: message.into(),
                },
            )))
            .expect("attempting to log on closed channel");
    }

    /// Send a message to the client.
    pub fn message(&mut self, ty: MessageType, message: impl Into<String>) {
        self.sender
            .send(Message::Notification(Notification::new(
                notification::ShowMessage::METHOD.to_string(),
                ShowMessageParams {
                    typ: ty,
                    message: message.into(),
                },
            )))
            .expect("attempting to send a message on a closed channel");
    }

    pub fn notify<N: notification::Notification>(&mut self, params: N::Params) {
        self.sender
            .send(Message::Notification(Notification::new(
                N::METHOD.to_string(),
                params,
            )))
            .expect("attempting to send notification on closed channel");
    }
}
