use zippy_common::messages::{Code, MessageContainer, MessageMaker, NoteKind};
use zippy_common::text;

use crate::messages::ClarifyMessages;

impl<C: MessageContainer> ClarifyMessages for MessageMaker<C> {
    fn incompatible_instances(&mut self) {
        let message = self.error(Code::TypeError)
            .with_title(text!["incompatible trait instances"])
            .with_note(NoteKind::Note, text![
                "although these appear to have the same trait type, they are actually different concrete types"
            ]);
        self.add(message);
    }

    fn recursive_instance(&mut self) {
        let message = self
            .error(Code::TypeError)
            .with_title(text!["recursive trait instance"]);
        self.add(message);
    }
}
