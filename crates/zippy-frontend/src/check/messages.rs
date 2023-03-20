use zippy_common::messages::{Code, MessageContainer, MessageMaker, NoteKind};
use zippy_common::text;

use crate::messages::TypeMessages;

impl<C: MessageContainer> TypeMessages for MessageMaker<C> {
    fn ambiguous(&mut self) {
        let message = self
            .error(Code::TypeError)
            .with_title(text!["too few constraints to determine types"]);
        self.add(message);
    }

    fn inequal_types(&mut self) {
        let message = self
            .error(Code::TypeError)
            .with_title(text!["inequal types"]);
        self.add(message);
    }

    fn missing_field(&mut self, name: &str) {
        let message = self
            .error(Code::TypeError)
            .with_title(text!["missing a field ", (code name)]);
        self.add(message);
    }

    fn not_a_trait(&mut self) {
        let message = self
            .error(Code::TypeError)
            .with_title(text!["cannot get a field from a non-trait type"]);
        self.add(message);
    }

    fn not_unitlike(&mut self) {
        let message = self
            .error(Code::TypeError)
            .with_title(text!["expected a unit type"])
            .with_note(
                NoteKind::Note,
                text!["only types that contain a single value can be used in a unit context"],
            );
        self.add(message);
    }

    fn not_numeric(&mut self) {
        let message = self
            .error(Code::TypeError)
            .with_title(text!["expected a numeric type"]);
        self.add(message);
    }

    fn not_textual(&mut self) {
        let message = self
            .error(Code::TypeError)
            .with_title(text!["expected a string type"]);
        self.add(message);
    }

    fn no_such_field(&mut self, name: &str) {
        let message = self
            .error(Code::NameError)
            .with_title(text!["no field with name ", (code name)]);
        self.add(message);
    }

    fn recursive_type(&mut self) {
        let message = self
            .error(Code::TypeError)
            .with_title(text!["occurs check failure"]);
        self.add(message);
    }
}
