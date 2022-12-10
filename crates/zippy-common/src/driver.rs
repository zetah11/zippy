use crate::message::Messages;

pub trait Driver {
    fn report(&mut self, messages: Messages);

    fn report_eval(&mut self, at: String);
    fn done_eval(&mut self);

    fn entry_name(&mut self) -> Option<String> {
        None
    }

    fn eval_amount(&mut self) -> EvalAmount {
        EvalAmount::Full
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum EvalAmount {
    Full,
    Types,
    None,
}
