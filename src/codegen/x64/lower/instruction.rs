use super::super::repr::{Instruction, Operand};

pub fn insert_move(within: &mut Vec<Instruction>, target: Operand, value: Operand) {
    if target != value {
        within.push(Instruction::Mov(target, value));
    }
}
