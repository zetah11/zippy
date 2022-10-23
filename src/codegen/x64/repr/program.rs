use std::collections::HashMap;

use super::names::Name;
use super::procedure::Procedure;

#[derive(Debug)]
pub struct Program {
    pub procedures: HashMap<Name, Procedure>,
}
