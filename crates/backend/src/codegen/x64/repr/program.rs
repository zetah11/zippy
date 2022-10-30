use super::names::{Name, Names};
use super::procedure::Procedure;

#[derive(Debug)]
pub struct Program {
    pub procedures: Vec<(Name, Procedure)>,
    pub names: Names,
}
