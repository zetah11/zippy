use common::lir::{self, BlockId, Target, Value};
use common::names::Name;
use iced_x86::code_asm::{gpr64, ptr, rax, AsmMemoryOperand, AsmRegister64};
use iced_x86::Register;

use super::{regid_to_reg, Lowerer};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Operand {
    Gpr64(AsmRegister64),
    Memory(AsmMemoryOperand),
    Label(Name),
    Block(BlockId),
    Integer(i64),
}

impl TryFrom<Register> for Operand {
    type Error = ();

    fn try_from(register: Register) -> Result<Self, Self::Error> {
        if let Some(reg) = gpr64::get_gpr64(register) {
            return Ok(Self::Gpr64(reg));
        }

        Err(())
    }
}

impl Lowerer<'_> {
    pub fn target_operand(&self, target: &Target) -> Option<Operand> {
        match target {
            Target::Register(lir::Register::Physical(id)) => {
                Operand::try_from(regid_to_reg(*id)).ok()
            }
            Target::Register(lir::Register::Frame(..)) => todo!(),
            Target::Register(lir::Register::Virtual(_)) => unreachable!(),

            Target::Name(name) => Some(Operand::Label(*name)),
        }
    }

    pub fn value_operand(&self, value: &Value) -> Option<Operand> {
        match value {
            Value::Register(lir::Register::Physical(id)) => {
                Operand::try_from(regid_to_reg(*id)).ok()
            }
            Value::Register(lir::Register::Frame(..)) => {
                todo!()
            }
            Value::Register(lir::Register::Virtual(_)) => unreachable!(),

            Value::Name(name) => Some(Operand::Label(*name)),

            Value::Integer(i) => Some(Operand::Integer(*i)),
        }
    }

    pub fn asm_call(&mut self, value: Operand) {
        match value {
            Operand::Gpr64(source) => self.asm.call(source).unwrap(),
            Operand::Memory(source) => self.asm.call(source).unwrap(),
            Operand::Label(source) => {
                let source = self.label(source);
                self.asm.call(source).unwrap();
            }
            Operand::Block(source) => {
                let source = self.block_label(source);
                self.asm.call(source).unwrap();
            }

            _ => unreachable!("invalid opcode/operand combination for call"),
        }
    }

    pub fn asm_cmp(&mut self, left: Operand, right: Operand) {
        match (left, right) {
            (Operand::Gpr64(left), Operand::Gpr64(right)) => self.asm.cmp(left, right).unwrap(),
            (Operand::Gpr64(left), Operand::Memory(right)) => self.asm.cmp(left, right).unwrap(),
            (Operand::Gpr64(left), Operand::Integer(i)) => match i32::try_from(i) {
                Ok(i) => self.asm.cmp(left, i).unwrap(),
                _ => {
                    self.asm.mov(rax, i).unwrap();
                    self.asm.cmp(left, rax).unwrap();
                }
            },

            (Operand::Memory(left), Operand::Gpr64(right)) => self.asm.cmp(left, right).unwrap(),
            (Operand::Memory(left), Operand::Integer(i)) => {
                match (i32::try_from(i), u32::try_from(i)) {
                    (Ok(i), _) => self.asm.cmp(left, i).unwrap(),
                    (_, Ok(i)) => self.asm.cmp(left, i).unwrap(),
                    (..) => {
                        self.asm.mov(rax, i).unwrap();
                        self.asm.cmp(left, rax).unwrap();
                    }
                }
            }

            _ => unreachable!("invalid opcode/operand combination for cmp"),
        }
    }

    pub fn asm_je(&mut self, value: Operand) {
        match value {
            Operand::Label(source) => {
                let source = self.label(source);
                self.asm.je(source).unwrap();
            }
            Operand::Block(source) => {
                let source = self.block_label(source);
                self.asm.je(source).unwrap();
            }

            _ => unreachable!("invalid opcode/operand combination for je"),
        }
    }

    pub fn asm_jg(&mut self, value: Operand) {
        match value {
            Operand::Label(source) => {
                let source = self.label(source);
                self.asm.jg(source).unwrap();
            }
            Operand::Block(source) => {
                let source = self.block_label(source);
                self.asm.jg(source).unwrap();
            }

            _ => unreachable!("invalid opcode/operand combination for je"),
        }
    }

    pub fn asm_jge(&mut self, value: Operand) {
        match value {
            Operand::Label(source) => {
                let source = self.label(source);
                self.asm.jge(source).unwrap();
            }
            Operand::Block(source) => {
                let source = self.block_label(source);
                self.asm.jge(source).unwrap();
            }

            _ => unreachable!("invalid opcode/operand combination for je"),
        }
    }

    pub fn asm_jl(&mut self, value: Operand) {
        match value {
            Operand::Label(source) => {
                let source = self.label(source);
                self.asm.jl(source).unwrap();
            }
            Operand::Block(source) => {
                let source = self.block_label(source);
                self.asm.jl(source).unwrap();
            }

            _ => unreachable!("invalid opcode/operand combination for je"),
        }
    }

    pub fn asm_jle(&mut self, value: Operand) {
        match value {
            Operand::Label(source) => {
                let source = self.label(source);
                self.asm.jle(source).unwrap();
            }
            Operand::Block(source) => {
                let source = self.block_label(source);
                self.asm.jle(source).unwrap();
            }

            _ => unreachable!("invalid opcode/operand combination for je"),
        }
    }

    pub fn asm_jmp(&mut self, value: Operand) {
        match value {
            Operand::Gpr64(source) => self.asm.jmp(source).unwrap(),
            Operand::Memory(source) => self.asm.jmp(source).unwrap(),
            Operand::Label(source) => {
                let source = self.label(source);
                self.asm.jmp(source).unwrap();
            }
            Operand::Block(source) => {
                let source = self.block_label(source);
                self.asm.jmp(source).unwrap();
            }

            _ => unreachable!("invalid opcode/operand combination for jmp"),
        }
    }

    pub fn asm_jne(&mut self, value: Operand) {
        match value {
            Operand::Label(source) => {
                let source = self.label(source);
                self.asm.jne(source).unwrap();
            }
            Operand::Block(source) => {
                let source = self.block_label(source);
                self.asm.jne(source).unwrap();
            }

            _ => unreachable!("invalid opcode/operand combination for je"),
        }
    }

    pub fn asm_leave(&mut self) {
        self.asm.leave().unwrap();
    }

    pub fn asm_mov(&mut self, target: Operand, source: Operand) {
        match (target, source) {
            (Operand::Gpr64(target), Operand::Gpr64(source)) => {
                self.asm.mov(target, source).unwrap()
            }
            (Operand::Gpr64(target), Operand::Memory(source)) => {
                self.asm.mov(target, source).unwrap()
            }
            (Operand::Gpr64(target), Operand::Integer(source)) => {
                self.asm.mov(target, source).unwrap()
            }
            (Operand::Gpr64(target), Operand::Label(source)) => {
                let source = self.label(source);
                self.asm.lea(target, ptr(source)).unwrap();
            }
            (Operand::Gpr64(target), Operand::Block(source)) => {
                let source = self.block_label(source);
                self.asm.lea(target, ptr(source)).unwrap();
            }

            (Operand::Memory(target), Operand::Gpr64(source)) => {
                self.asm.mov(target, source).unwrap()
            }
            (Operand::Memory(target), Operand::Integer(source)) => {
                match (i32::try_from(source), u32::try_from(source)) {
                    (Ok(i), _) => self.asm.mov(target, i).unwrap(),
                    (_, Ok(i)) => self.asm.mov(target, i).unwrap(),
                    (..) => {
                        self.asm.mov(rax, source).unwrap();
                        self.asm.mov(target, rax).unwrap();
                    }
                }
            }

            _ => unreachable!("invalid opcode/operand combination for mov"),
        }
    }

    pub fn asm_pop(&mut self, target: Operand) {
        match target {
            Operand::Gpr64(target) => self.asm.pop(target).unwrap(),
            Operand::Memory(target) => self.asm.pop(target).unwrap(),

            _ => unreachable!("invalid opcode/operand combination for pop"),
        }
    }

    pub fn asm_push(&mut self, value: Operand) {
        match value {
            Operand::Gpr64(source) => self.asm.push(source).unwrap(),
            Operand::Memory(source) => self.asm.push(source).unwrap(),
            Operand::Integer(i) => match (i32::try_from(i), u32::try_from(i)) {
                (Ok(i), _) => self.asm.push(i).unwrap(),
                (_, Ok(i)) => self.asm.push(i).unwrap(),
                (..) => {
                    self.asm.mov(rax, i).unwrap();
                    self.asm.push(rax).unwrap();
                }
            },
            Operand::Label(source) => {
                let source = self.label(source);
                self.asm.lea(rax, ptr(source)).unwrap();
                self.asm.push(rax).unwrap();
            }
            Operand::Block(source) => {
                let source = self.block_label(source);
                self.asm.lea(rax, ptr(source)).unwrap();
                self.asm.push(rax).unwrap();
            }
        }
    }

    pub fn asm_ret(&mut self) {
        self.asm.ret().unwrap();
    }

    pub fn asm_ret1(&mut self, by: u32) {
        self.asm.ret_1(by).unwrap();
    }
}
