use crate::message::Messages;

pub trait Driver {
    fn report(&mut self, messages: Messages);
    fn report_eval(&mut self, at: String);
}
