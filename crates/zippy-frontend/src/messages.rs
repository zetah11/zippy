use zippy_common::messages::{Message, MessageContainer, Messages};

use crate::Db;

pub trait ParseMessages {
    /// Expected an expression.
    fn expected_expression(&mut self);

    /// Expected an item.
    fn expected_item(&mut self);

    /// Expected a pattern.
    fn expected_pattern(&mut self);

    /// Expected a type.
    fn expected_type(&mut self);

    /// The indentation is not correct
    fn indent_error(&mut self, expected: usize, actual: usize);

    /// An unmatched parenthesis was encountered.
    fn unclosed_parenthesis(&mut self);

    /// An invalid token was encountered
    fn unexpected_token(&mut self);
}

impl MessageContainer for &'_ dyn Db {
    fn push(&mut self, message: Message) {
        let db = <dyn Db as salsa::DbWithJar<zippy_common::Jar>>::as_jar_db(*self);
        Messages::push(db, message)
    }
}
