use zippy_common::messages::{Code, MessageContainer, MessageMaker};
use zippy_common::text;

use crate::messages::ParseMessages;

impl<C: MessageContainer> ParseMessages for MessageMaker<C> {
    fn unexpected_token(&mut self) {
        let message = self
            .error(Code::SyntaxError)
            .with_title(text!["unexpected token"]);
        self.add(message)
    }
}
