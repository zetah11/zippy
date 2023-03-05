use zippy_common::messages::{Code, MessageContainer, MessageMaker, NoteKind};
use zippy_common::names::Name;
use zippy_common::source::Span;
use zippy_common::text;

use crate::messages::NameMessages;

impl<C: MessageContainer> NameMessages for MessageMaker<C> {
    fn bare_import_unsupported(&mut self) {
        let message = self
            .error(Code::SyntaxError)
            .with_title(text!["bare imports are currently unsupported"])
            .with_note(
                NoteKind::Note,
                text!["an import must have some item name to import from"],
            );
        self.add(message)
    }

    fn duplicate_definition(&mut self, name: Name, previous: Span) {
        let labels = vec![(previous, text!["previous definition here"])];

        let message = self
            .error(Code::DeclarationError)
            .with_title(text![(name name), " is defined multiple times"])
            .with_labels(labels);
        self.add(message);
    }

    fn unresolved_name(&mut self, name: &str) {
        let message = self
            .error(Code::NameError)
            .with_title(text!["no item named ", (code name)]);
        self.add(message);
    }
}
