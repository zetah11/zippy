use zippy_common::messages::{Code, MessageContainer, MessageMaker};
use zippy_common::names::Name;
use zippy_common::source::Span;
use zippy_common::text;

use crate::messages::NameMessages;

impl<C: MessageContainer> NameMessages for MessageMaker<C> {
    fn duplicate_definition(&mut self, name: Option<Name>, previous: Span) {
        let title = if let Some(name) = name {
            text![(name name), " is defined multiple times"]
        } else {
            text!["item is defined several times"]
        };

        let labels = vec![(previous, text!["previous definition here"])];

        let message = self
            .error(Code::DeclarationError)
            .with_title(title)
            .with_labels(labels);
        self.add(message);
    }
}
