use super::Register;
use crate::names::Name;

#[derive(Clone, Debug)]
pub enum Value {
    Integer(i64),
    Register(Register),
    Name(Name),
}

#[derive(Clone, Debug)]
pub enum Target {
    Register(Register),
    Name(Name),
}
