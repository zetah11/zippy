use zippy_common::messages::{Code, MessageContainer, MessageMaker, NoteKind};
use zippy_common::text;

use crate::messages::ParseMessages;

impl<C: MessageContainer> ParseMessages for MessageMaker<C> {
    fn expected_expression(&mut self) {
        let message = self
            .error(Code::SyntaxError)
            .with_title(text!["expected an expression"]);
        self.add(message);
    }

    fn expected_item(&mut self) {
        let message = self
            .error(Code::SyntaxError)
            .with_title(text!["expected an item"]);
        self.add(message);
    }

    fn expected_pattern(&mut self) {
        let message = self
            .error(Code::SyntaxError)
            .with_title(text!["expected a pattern"]);
        self.add(message);
    }

    fn expected_type(&mut self) {
        let message = self
            .error(Code::SyntaxError)
            .with_title(text!["expected a type"]);
        self.add(message);
    }

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

    fn unclosed_parenthesis(&mut self) {
        let message = self
            .error(Code::SyntaxError)
            .with_title(text!["unclosed parenthesis"]);
        self.add(message);
    }

    fn unexpected_token(&mut self) {
        let message = self
            .error(Code::SyntaxError)
            .with_title(text!["unexpected token"]);
        self.add(message)
    }
}
