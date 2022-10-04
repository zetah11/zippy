use crate::message::Messages;

pub trait Driver {
    fn report(&mut self, messages: Messages);
}
