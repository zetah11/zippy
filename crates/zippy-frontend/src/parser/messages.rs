use zippy_common::messages::{Code, MessageContainer, MessageMaker, NoteKind};
use zippy_common::text;

use crate::messages::ParseMessages;

impl<C: MessageContainer> ParseMessages for MessageMaker<C> {
    fn indent_error(&mut self, expected: usize, actual: usize) {
        let help = format!(
            "try removing {} space{}",
            actual - expected,
            if actual - expected == 1 { "" } else { "s" }
        );

        let notes = vec![
            (NoteKind::Note, text![
                "when the indentation level decreases, it must match some previously seen indentation level"
            ]),
            (NoteKind::Help, text![help]),
        ];

        let message = self
            .error(Code::SyntaxError)
            .with_title(text!["indentation is not correct"])
            .with_notes(notes);
        self.add(message);
    }

    fn unexpected_token(&mut self) {
        let message = self
            .error(Code::SyntaxError)
            .with_title(text!["unexpected token"]);
        self.add(message)
    }
}
