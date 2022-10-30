use super::operand::Operand;

#[derive(Clone, Debug)]
pub enum Instruction {
    Call(Operand),
    Cmp(Operand, Operand),
    Jump(Operand),
    Leave,
    Mov(Operand, Operand),
    Push(Operand),
    Pop(Operand),
    Ret,
    Sub(Operand, Operand),
    Syscall,
}
