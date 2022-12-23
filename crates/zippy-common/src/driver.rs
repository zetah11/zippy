use crate::message::Messages;

pub trait Driver {
    fn report(&mut self, messages: Messages);

    fn report_eval(&mut self, at: String);
    fn done_eval(&mut self);

    /// Output the IR for the given stage. The IR string is taken as a function,
    /// since generating it would usually be wasteful.
    fn output_ir(&mut self, at: IrOutput, data: impl FnOnce() -> String);

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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum IrOutput {
    Mir(&'static str),
}
