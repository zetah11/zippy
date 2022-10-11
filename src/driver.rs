use crate::message::Messages;

pub trait Driver {
    fn report(&mut self, messages: Messages);

    fn entry_name(&mut self) -> Option<String> {
        None
    }

    fn report_eval(&mut self, at: String);
    fn done_eval(&mut self);
}
