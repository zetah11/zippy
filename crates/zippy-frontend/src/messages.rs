use zippy_common::messages::{Message, MessageContainer, Messages};

use crate::Db;

pub trait ParseMessages {
    /// The indentation is not correct
    fn indent_error(&mut self, expected: usize, actual: usize);

    /// An invalid token was encountered
    fn unexpected_token(&mut self);
}

impl MessageContainer for &'_ dyn Db {
    fn push(&mut self, message: Message) {
        let db = <dyn Db as salsa::DbWithJar<zippy_common::Jar>>::as_jar_db(*self);
        Messages::push(db, message)
    }
}
