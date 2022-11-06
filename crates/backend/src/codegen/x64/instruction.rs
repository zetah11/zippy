use common::lir::{self, BlockId, Target, Value};
use common::names::Name;
use iced_x86::code_asm::{gpr64, ptr, rax, AsmMemoryOperand, AsmRegister64};
use iced_x86::Register;

use super::{regid_to_reg, Error, Lowerer};

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

    pub fn asm_call(&mut self, value: Operand) -> Result<(), Error> {
        match value {
            Operand::Gpr64(source) => self.asm.call(source)?,
            Operand::Memory(source) => self.asm.call(source)?,
            Operand::Label(source) => {
                let source = self.label(source);
                self.asm.call(source)?;
            }
            Operand::Block(source) => {
                let source = self.block_label(source);
                self.asm.call(source)?;
            }

            _ => unreachable!("invalid opcode/operand combination for call"),
        }

        Ok(())
    }

    pub fn asm_cmp(&mut self, left: Operand, right: Operand) -> Result<(), Error> {
        match (left, right) {
            (Operand::Gpr64(left), Operand::Gpr64(right)) => self.asm.cmp(left, right)?,
            (Operand::Gpr64(left), Operand::Memory(right)) => self.asm.cmp(left, right)?,
            (Operand::Gpr64(left), Operand::Integer(i)) => match i32::try_from(i) {
                Ok(i) => self.asm.cmp(left, i)?,
                _ => {
                    self.asm.mov(rax, i)?;
                    self.asm.cmp(left, rax)?;
                }
            },

            (Operand::Memory(left), Operand::Gpr64(right)) => self.asm.cmp(left, right)?,
            (Operand::Memory(left), Operand::Integer(i)) => {
                match (i32::try_from(i), u32::try_from(i)) {
                    (Ok(i), _) => self.asm.cmp(left, i)?,
                    (_, Ok(i)) => self.asm.cmp(left, i)?,
                    (..) => {
                        self.asm.mov(rax, i)?;
                        self.asm.cmp(left, rax)?;
                    }
                }
            }

            _ => unreachable!("invalid opcode/operand combination for cmp"),
        }

        Ok(())
    }

    pub fn asm_je(&mut self, value: Operand) -> Result<(), Error> {
        match value {
            Operand::Label(source) => {
                let source = self.label(source);
                self.asm.je(source)?;
            }
            Operand::Block(source) => {
                let source = self.block_label(source);
                self.asm.je(source)?;
            }

            _ => unreachable!("invalid opcode/operand combination for je"),
        }

        Ok(())
    }

    pub fn asm_jg(&mut self, value: Operand) -> Result<(), Error> {
        match value {
            Operand::Label(source) => {
                let source = self.label(source);
                self.asm.jg(source)?;
            }
            Operand::Block(source) => {
                let source = self.block_label(source);
                self.asm.jg(source)?;
            }

            _ => unreachable!("invalid opcode/operand combination for je"),
        }

        Ok(())
    }

    pub fn asm_jge(&mut self, value: Operand) -> Result<(), Error> {
        match value {
            Operand::Label(source) => {
                let source = self.label(source);
                self.asm.jge(source)?;
            }
            Operand::Block(source) => {
                let source = self.block_label(source);
                self.asm.jge(source)?;
            }

            _ => unreachable!("invalid opcode/operand combination for je"),
        }

        Ok(())
    }

    pub fn asm_jl(&mut self, value: Operand) -> Result<(), Error> {
        match value {
            Operand::Label(source) => {
                let source = self.label(source);
                self.asm.jl(source)?;
            }
            Operand::Block(source) => {
                let source = self.block_label(source);
                self.asm.jl(source)?;
            }

            _ => unreachable!("invalid opcode/operand combination for je"),
        }

        Ok(())
    }

    pub fn asm_jle(&mut self, value: Operand) -> Result<(), Error> {
        match value {
            Operand::Label(source) => {
                let source = self.label(source);
                self.asm.jle(source)?;
            }
            Operand::Block(source) => {
                let source = self.block_label(source);
                self.asm.jle(source)?;
            }

            _ => unreachable!("invalid opcode/operand combination for je"),
        }

        Ok(())
    }

    pub fn asm_jmp(&mut self, value: Operand) -> Result<(), Error> {
        match value {
            Operand::Gpr64(source) => self.asm.jmp(source)?,
            Operand::Memory(source) => self.asm.jmp(source)?,
            Operand::Label(source) => {
                let source = self.label(source);
                self.asm.jmp(source)?;
            }
            Operand::Block(source) => {
                let source = self.block_label(source);
                self.asm.jmp(source)?;
            }

            _ => unreachable!("invalid opcode/operand combination for jmp"),
        }

        Ok(())
    }

    pub fn asm_jne(&mut self, value: Operand) -> Result<(), Error> {
        match value {
            Operand::Label(source) => {
                let source = self.label(source);
                self.asm.jne(source)?;
            }
            Operand::Block(source) => {
                let source = self.block_label(source);
                self.asm.jne(source)?;
            }

            _ => unreachable!("invalid opcode/operand combination for je"),
        }

        Ok(())
    }

    pub fn asm_leave(&mut self) -> Result<(), Error> {
        self.asm.leave()?;
        Ok(())
    }

    pub fn asm_mov(&mut self, target: Operand, source: Operand) -> Result<(), Error> {
        match (target, source) {
            (Operand::Gpr64(target), Operand::Gpr64(source)) => self.asm.mov(target, source)?,
            (Operand::Gpr64(target), Operand::Memory(source)) => self.asm.mov(target, source)?,
            (Operand::Gpr64(target), Operand::Integer(source)) => self.asm.mov(target, source)?,
            (Operand::Gpr64(target), Operand::Label(source)) => {
                let source = self.label(source);
                self.asm.lea(target, ptr(source))?;
            }
            (Operand::Gpr64(target), Operand::Block(source)) => {
                let source = self.block_label(source);
                self.asm.lea(target, ptr(source))?;
            }

            (Operand::Memory(target), Operand::Gpr64(source)) => self.asm.mov(target, source)?,
            (Operand::Memory(target), Operand::Integer(source)) => {
                match (i32::try_from(source), u32::try_from(source)) {
                    (Ok(i), _) => self.asm.mov(target, i)?,
                    (_, Ok(i)) => self.asm.mov(target, i)?,
                    (..) => {
                        self.asm.mov(rax, source)?;
                        self.asm.mov(target, rax)?;
                    }
                }
            }

            _ => unreachable!("invalid opcode/operand combination for mov"),
        }

        Ok(())
    }

    pub fn asm_pop(&mut self, target: Operand) -> Result<(), Error> {
        match target {
            Operand::Gpr64(target) => self.asm.pop(target)?,
            Operand::Memory(target) => self.asm.pop(target)?,

            _ => unreachable!("invalid opcode/operand combination for pop"),
        }

        Ok(())
    }

    pub fn asm_push(&mut self, value: Operand) -> Result<(), Error> {
        match value {
            Operand::Gpr64(source) => self.asm.push(source)?,
            Operand::Memory(source) => self.asm.push(source)?,
            Operand::Integer(i) => match (i32::try_from(i), u32::try_from(i)) {
                (Ok(i), _) => self.asm.push(i)?,
                (_, Ok(i)) => self.asm.push(i)?,
                (..) => {
                    self.asm.mov(rax, i)?;
                    self.asm.push(rax)?;
                }
            },
            Operand::Label(source) => {
                let source = self.label(source);
                self.asm.lea(rax, ptr(source))?;
                self.asm.push(rax)?;
            }
            Operand::Block(source) => {
                let source = self.block_label(source);
                self.asm.lea(rax, ptr(source))?;
                self.asm.push(rax)?;
            }
        }

        Ok(())
    }

    pub fn asm_ret(&mut self) -> Result<(), Error> {
        self.asm.ret()?;
        Ok(())
    }

    pub fn asm_ret1(&mut self, by: u32) -> Result<(), Error> {
        self.asm.ret_1(by)?;
        Ok(())
    }
}
