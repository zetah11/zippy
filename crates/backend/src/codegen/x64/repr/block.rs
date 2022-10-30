use super::instruction::Instruction;

#[derive(Debug)]
pub struct Block {
    pub insts: Vec<Instruction>,
}
