use super::names::Name;
use super::procedure::Procedure;

#[derive(Debug)]
pub struct Program {
    pub procedures: Vec<(Name, Procedure)>,
}
