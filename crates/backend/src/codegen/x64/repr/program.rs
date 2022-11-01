use super::names::{Name, Names};
use super::procedure::Procedure;

#[derive(Debug)]
pub struct Program {
    pub procedures: Vec<(Name, Procedure)>,
    pub constants: Vec<(Name, Vec<u8>)>,
    pub names: Names,
}
